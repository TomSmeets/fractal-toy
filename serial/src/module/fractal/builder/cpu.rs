use super::{TileRequest, TileType};
use crate::module::fractal::TEXTURE_SIZE;

use crate::math::*;

pub fn build(rq: TileRequest) -> Vec<u8> {
    let mut pixels = vec![0; TEXTURE_SIZE * TEXTURE_SIZE * 4];

    match rq.kind {
        TileType::Empty => {
            for y in 0..TEXTURE_SIZE {
                for x in 0..TEXTURE_SIZE {
                    let i = y * TEXTURE_SIZE + x;
                    if x <= 4 || y <= 4 || x >= TEXTURE_SIZE - 5 || y >= TEXTURE_SIZE - 5 {
                        pixels[i * 4 + 0] = 255;
                        pixels[i * 4 + 1] = 64;
                        pixels[i * 4 + 2] = 64;
                        pixels[i * 4 + 3] = 64;
                    } else {
                        let dx = x as i32 * 2 - TEXTURE_SIZE as i32;
                        let dy = y as i32 * 2 - TEXTURE_SIZE as i32;
                        let r = dx * dx + dy * dy;
                        let l = TEXTURE_SIZE as i32;
                        let c = if r < l * l { 255 } else { 0 };
                        pixels[i * 4 + 0] = 255;
                        pixels[i * 4 + 1] = c as u8;
                        pixels[i * 4 + 2] = (x * c / TEXTURE_SIZE) as u8;
                        pixels[i * 4 + 3] = (y * c / TEXTURE_SIZE) as u8;
                    }
                }
            }
        },
        TileType::Mandelbrot => {
            draw_mandel(rq, &mut pixels, |mut z, c| {
                z = cpx_sqr(z) + c;
                z
            });
        },
        TileType::ShipHybrid => {
            draw_mandel(rq, &mut pixels, |mut z, c| {
                z = cpx_abs(z);
                z = cpx_sqr(z) + c;
                z = cpx_sqr(z) + c;
                z
            });
        },
        _ => (),
    }
    pixels
}

// How about this? a thread pool generator
// NO, that won't work. how would you cancle queued jobs?
// struct ThreadedGen<G> {
//    gen: G,
// }

// TODO: profile!!
fn draw_mandel<F: Fn(V2, V2) -> V2 + Copy>(rq: TileRequest, pixels: &mut [u8], f: F) {
    let [offset_x, offset_y, zoom] = rq.pos.to_f64_with_padding();
    let offset = Vector2::new(offset_x, offset_y);

    let iterations = rq.iterations as u32;
    let inv_size = 1.0 / TEXTURE_SIZE as f64;
    let inv_iter = 1.0 / iterations as f64;

    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            let mut c0 = Vector2::new(x as f64, y as f64);

            // screen coords 0 - 1
            c0 *= inv_size;
            c0.y = 1.0 - c0.y;

            // -1 , 1
            c0 = zoom * c0 + offset;

            let itr = mandel(iterations, c0, f);

            let mut v = itr * inv_iter;
            v *= v;
            v = 1. - v;

            let rgb = hsv2rgb(itr as f64 / 32.0, v, v);
            let idx = x + y * TEXTURE_SIZE;
            pixels[idx * 4 + 0] = 255;
            pixels[idx * 4 + 1] = rgb[0];
            pixels[idx * 4 + 2] = rgb[1];
            pixels[idx * 4 + 3] = rgb[2];
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
fn mandel<F: Fn(V2, V2) -> V2>(max: u32, c: Vector2<f64>, f: F) -> f64 {
    let mut z = c;
    let mut n = 0;
    loop {
        z = f(z, c);
        if n == max {
            return max as f64;
        }

        if z.x * z.x + z.y * z.y > 4.0 {
            return n as f64;
        }

        n += 1;
    }
}

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
