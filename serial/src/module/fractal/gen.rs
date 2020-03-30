use super::{TilePos, TEXTURE_SIZE};
use crate::math::*;
use serde::{Deserialize, Serialize};

/// A service that can produce fractal image tiles. This trait should only
/// represent the method of computing and precision. Everything else is
/// determined by the arguments to the generate function.
///
/// TODO: This should eventually become a trait.
/// Future implementations could be:
///     TODO: cuda-float
///     TODO: cuda-double
///     TODO: opencl-float
///     TODO: opencl-double
///     TODO: cpu-float
///     TODO: cpu-double
///     TODO: sse2
///     TODO: avx
///
/// INFO:
///     cuda: cudaStreamQuery
/// TODO: ablility to save and load multiple locations
/// TODO: share saved location (via clipboard)
/// TODO: when loading a location, we could transition to it, by first zooming
///       out, then zooming in to that location
/// TODO: share sets of locations?
#[derive(Serialize, Deserialize)]
pub struct Gen {
    iterations: u32,
}

/// TileBuilder:  threaded, cuda, opencl
/// TileType:     Empty, Mandel, BurningShip
///
/// we have one queue, and ecach tilebuilder grabs from that queue
/// builders = [ CudaTileBuilder(Arc::clone(queue)),
/// ThrededBuidler(Arc::clone(queue)) ]
///
/// then the queue contains all information for poducing a tile.
/// like iteration count, fractal type, etc

fn hsv2rgb(hue: f64, sat: f64, val: f64) -> [u8; 3] {
    let hue = hue.fract();
    let hue = hue * 6.0;
    let part = hue as u32;
    let fract = hue - part as f64;

    // upper limit
    let max = 255.0 * val;
    // lower limit
    let min = 255.0 * val - 255.0 * val * sat;
    // increasing slope
    let inc = fract * max + (1.0 - fract) * min;
    // decreasing slope
    let dec = fract * min + (1.0 - fract) * max;

    // as u8
    let min = min as u8;
    let max = max as u8;
    let inc = inc as u8;
    let dec = dec as u8;
    match part {
        0 => [max, inc, min],
        1 => [dec, max, min],
        2 => [min, max, inc],
        3 => [min, dec, max],
        4 => [inc, min, max],
        5 => [max, min, dec],
        _ => [max, max, max],
    }
}

impl Gen {
    pub fn new() -> Gen {
        Gen { iterations: 256 }
    }

    /// This function should receive all required
    /// information to generate a reproducible fractal image
    ///
    /// TODO:
    /// the user should be able to change the algorithm.
    /// This could be achieved with decent performance by doing each stage on
    /// all pixels at the same time. this way the use of sse instructions
    /// can be maximized and the number of comparisons minimized
    /// Tile size should probably be configurable by the generator backend
    /// implementations. As different backends have different optimal tile
    /// sizes.
    /// TODO: don't return excact pixels, but the complex numbers and/or
    /// iterations
    /// TODO: This gen shold be initialized with fractal algorithm (and maybe
    /// colorcheme), so it can pre compile a program for it.
    pub fn generate(&self, tile: TilePos) -> Vec<u8> {
        let mut pixels = vec![0; (TEXTURE_SIZE * TEXTURE_SIZE * 4) as usize];

        let [x, y, size] = tile.to_f64_with_padding();
        let mut center = Vector2::new(x, y) * 4.0;

        if true {
            draw_mandel(
                self.iterations,
                &mut pixels,
                TEXTURE_SIZE as u32,
                TEXTURE_SIZE as u32,
                size * 4.0,
                center,
            );
        }

        if false {
            for y in 0..TEXTURE_SIZE {
                for x in 0..TEXTURE_SIZE {
                    if x <= 4 || y <= 4 || x >= TEXTURE_SIZE - 5 || y >= TEXTURE_SIZE - 5 {
                        pixels[0 + y * TEXTURE_SIZE * 4 + x * 4] = 64;
                        pixels[1 + y * TEXTURE_SIZE * 4 + x * 4] = 64;
                        pixels[2 + y * TEXTURE_SIZE * 4 + x * 4] = 64;
                        pixels[3 + y * TEXTURE_SIZE * 4 + x * 4] = 64;
                    }
                }
            }
        }

        pixels
    }
}

// How about this? a thread pool generator
// NO, that won't work. how would you cancle queued jobs?
// struct ThreadedGen<G> {
//    gen: G,
// }

// TODO: profile!!
fn draw_mandel(
    iterations: u32,
    pixels: &mut [u8],
    w: u32,
    h: u32,
    zoom: f64,
    offset: Vector2<f64>,
) {
    let inv_w = 1.0 / w as f64;
    let inv_h = 1.0 / h as f64;
    let inv_iter = 1.0 / iterations as f64;

    for y in 0..h {
        for x in 0..w {
            let mut c0 = Vector2::new(x as f64, y as f64);

            // screen coords 0 - 1
            c0.x *= inv_w;
            c0.y *= inv_h;
            c0.y = 1.0 - c0.y;

            // -1 , 1
            c0 = zoom * c0 + offset;

            let itr = mandel(iterations, c0);

            let mut v = itr * inv_iter;
            v *= v;
            v = 1. - v;

            let rgb = hsv2rgb(itr as f64 / 32.0, v, v);
            pixels[(0 + (x + y * w) * 4) as usize] = 255;
            pixels[(1 + (x + y * w) * 4) as usize] = rgb[0];
            pixels[(2 + (x + y * w) * 4) as usize] = rgb[1];
            pixels[(3 + (x + y * w) * 4) as usize] = rgb[2];
        }
    }
}

fn cpx_mul(a: V2, b: V2) -> V2 {
    V2 {
        x: a.x * b.x - a.y * b.y,
        y: a.x * b.y + a.y * b.x,
    }
}

fn cpx_sqr(a: V2) -> V2 {
    V2 {
        x: a.x * a.x - a.y * a.y,
        y: 2.0 * a.x * a.y,
    }
}

fn cpx_abs(a: V2) -> V2 {
    V2 {
        x: a.x.abs(),
        y: -a.y.abs(),
    }
}

// some cool algorithms
// nice: ((|re| + |im|i)^2 + c)^3 + c
fn mandel(max: u32, c: Vector2<f64>) -> f64 {
    let mut z = c;
    let mut n = 0;
    loop {
        z = cpx_abs(z);
        z = cpx_sqr(z) + c;
        z = cpx_sqr(z) + c;

        if n == max {
            return max as f64;
        }

        if z.x * z.x + z.y * z.y > 4.0 {
            return n as f64;
        }

        n += 1;
    }
}
