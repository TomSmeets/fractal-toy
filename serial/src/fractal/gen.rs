use super::TilePos;

use super::TEXTURE_SIZE;
use crate::math::*;
use ::palette::*;

pub struct Gen {}

impl Gen {
    pub fn generate(tile: TilePos) -> Vec<u8> {
        draw_tile(tile)
    }
}

pub fn draw_tile(p: TilePos) -> Vec<u8> {
    let mut pixels = vec![0; (TEXTURE_SIZE * TEXTURE_SIZE * 4) as usize];

    let [x, y, size] = p.to_f64();
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
        z = cpx_sqr(z) + c;

        if n == max {
            return max;
        }

        if z.x * z.x + z.y * z.y > 4.0 {
            return n;
        }

        n += 1;
    }
}
