use super::TilePos;

use super::TEXTURE_SIZE;
use crate::math::*;
use ::palette::*;

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
pub struct Gen {}

impl Gen {
    pub fn new() -> Gen {
        Gen {}
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

        let [x, y, size] = tile.to_f64();
        let center = Vector2::new(x, y) * 4.0 - Vector2::new(2.0, 2.0);
        draw_mandel(
            &mut pixels,
            TEXTURE_SIZE as u32,
            TEXTURE_SIZE as u32,
            size * 4.0,
            center,
        );

        pixels
    }
}

// How about this? a thread pool generator
// NO, that won't work. how would you cancle queued jobs?
// struct ThreadedGen<G> {
//    gen: G,
// }

// TODO: profile!!
fn draw_mandel(pixels: &mut [u8], w: u32, h: u32, zoom: f64, offset: Vector2<f64>) {
    for y in 0..h {
        for x in 0..w {
            let mut c0 = Vector2::new(x as f64, y as f64);

            // screen coords 0 - 1
            c0.x /= w as f64;
            c0.y /= h as f64;
            c0.y = 1.0 - c0.y;

            // -1 , 1
            c0 = zoom * c0 + offset;

            let itr = mandel(256, c0);

            let mut v = itr as f64 / 256.0;
            v *= v;
            v = 1. - v;

            let hsv = Hsv::new(itr as f64 / 32.0 * 360., v, v);
            let rgb = Srgb::from(hsv).into_linear();

            pixels[(0 + (x + y * w) * 4) as usize] = 255;
            pixels[(1 + (x + y * w) * 4) as usize] = (rgb.red * 255.) as u8;
            pixels[(2 + (x + y * w) * 4) as usize] = (rgb.green * 255.) as u8;
            pixels[(3 + (x + y * w) * 4) as usize] = (rgb.blue * 255.) as u8;
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

fn mandel(max: i32, c: Vector2<f64>) -> i32 {
    let mut z = c;
    let mut n = 0;
    loop {
        // z.x = z.x.abs();
        // z.y = z.y.abs();
        z = cpx_sqr(z) + c;
        // z = cpx_mul(cpx_sqr(z), z) + c;

        if n == max {
            return max;
        }

        if z.x * z.x + z.y * z.y > 4.0 {
            return n;
        }

        n += 1;
    }
}
