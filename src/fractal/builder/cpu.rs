use super::{TileRequest, TileType};
use crate::fractal::TEXTURE_SIZE;

use crate::math::*;

pub fn build(rq: TileRequest) -> Vec<u8> {
    let mut pixels = vec![0; TEXTURE_SIZE * TEXTURE_SIZE * 4];

    match rq.params.kind {
        TileType::Empty => {
            for y in 0..TEXTURE_SIZE {
                for x in 0..TEXTURE_SIZE {
                    let i = y * TEXTURE_SIZE + x;
                    if x <= 4 || y <= 4 || x >= TEXTURE_SIZE - 5 || y >= TEXTURE_SIZE - 5 {
                        unsafe {
                            *pixels.get_unchecked_mut(i * 4 + 0) = 64;
                            *pixels.get_unchecked_mut(i * 4 + 1) = 64;
                            *pixels.get_unchecked_mut(i * 4 + 2) = 64;
                            *pixels.get_unchecked_mut(i * 4 + 3) = 255;
                        }
                    } else {
                        let dx = x as i32 * 2 - TEXTURE_SIZE as i32;
                        let dy = y as i32 * 2 - TEXTURE_SIZE as i32;
                        let r = dx * dx + dy * dy;
                        let l = TEXTURE_SIZE as i32;
                        let c = if r < l * l { 255 } else { 0 };
                        unsafe {
                            *pixels.get_unchecked_mut(i * 4 + 0) = c as u8;
                            *pixels.get_unchecked_mut(i * 4 + 1) = (x * c / TEXTURE_SIZE) as u8;
                            *pixels.get_unchecked_mut(i * 4 + 2) = (y * c / TEXTURE_SIZE) as u8;
                            *pixels.get_unchecked_mut(i * 4 + 3) = 255;
                        }
                    }
                }
            }
        },
        TileType::Mandelbrot => {
            draw_mandel(1.0, rq, &mut pixels, |mut z, c| {
                z = cpx_sqr(z) + c;
                z
            });
        },
        TileType::BurningShip => {
            draw_mandel(1.0, rq, &mut pixels, |mut z, c| {
                z = cpx_abs(z);
                z = cpx_sqr(z) + c;
                z
            });
        },
        // cube = 1.5, sqr = 1.0
        TileType::ShipHybrid => {
            draw_mandel(2.5, rq, &mut pixels, |mut z, c| {
                z = cpx_cube(z) + c; // 1.5
                z = cpx_abs(z);
                z = cpx_sqr(z) + c; // 1.0
                z
            });
        },
    }
    pixels
}

fn draw_mandel<F: Fn(V2, V2) -> V2 + Copy>(inc: f64, rq: TileRequest, pixels: &mut [u8], f: F) {
    let [offset_x, offset_y, zoom] = rq.pos.to_f64_with_padding();
    let offset = Vector2::new(offset_x, offset_y);

    let iterations = rq.params.iterations as u32;
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

            let itr = mandel(inc, iterations, c0, f);

            let mut v = itr * inv_iter;
            v *= v;
            v = 1. - v;

            let rgb = hsv2rgb(itr as f64 / 64.0, v, v);
            let idx = x + y * TEXTURE_SIZE;
            unsafe {
                *pixels.get_unchecked_mut(idx * 4 + 0) = rgb[0];
                *pixels.get_unchecked_mut(idx * 4 + 1) = rgb[1];
                *pixels.get_unchecked_mut(idx * 4 + 2) = rgb[2];
                *pixels.get_unchecked_mut(idx * 4 + 3) = 255;
            }
        }
    }
}

fn cpx_mul(a: V2, b: V2) -> V2 {
    V2 {
        x: a.x * b.x - a.y * b.y,
        y: a.x * b.y + a.y * b.x,
    }
}

fn cpx_cube(a: V2) -> V2 {
    cpx_mul(cpx_sqr(a), a)
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
fn mandel<F: Fn(V2, V2) -> V2>(inc: f64, max: u32, c: V2, f: F) -> f64 {
    let mut z = V2::zero();
    let mut n = 0.0;
    let max = max as f64;
    loop {
        z = f(z, c);

        if n >= max {
            return max;
        }

        if z.x * z.x + z.y * z.y > 64.0 * 64.0 {
            // mandel
            return n as f64 - (z.x * z.x + z.y * z.y).log2().log2() + 4.0;
        }

        n += inc;
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