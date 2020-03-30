use super::{TileRequest, TileType};
use crate::module::fractal::TEXTURE_SIZE;

use crate::math::*;

pub fn build(rq: TileRequest) -> Vec<u8> {
    let mut pixels = Vec::with_capacity(TEXTURE_SIZE * TEXTURE_SIZE * 4);

    let [x, y, size] = rq.pos.to_f64_with_padding();
    let mut center = Vector2::new(x, y) * 4.0;

    match rq.kind {
        TileType::Empty => {
            for y in 0..TEXTURE_SIZE {
                for x in 0..TEXTURE_SIZE {
                    if x <= 4 || y <= 4 || x >= TEXTURE_SIZE - 5 || y >= TEXTURE_SIZE - 5 {
                        pixels.push(64);
                        pixels.push(64);
                        pixels.push(64);
                        pixels.push(64);
                    } else {
                        pixels.push(0);
                        pixels.push(0);
                        pixels.push(0);
                        pixels.push(0);
                    }
                }
            }
        },
        TileType::Mandelbrot => {},
        _ => (),
    }

    assert_eq!(pixels.len(), TEXTURE_SIZE * TEXTURE_SIZE * 4);
    pixels
}
