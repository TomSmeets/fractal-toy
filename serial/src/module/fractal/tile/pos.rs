use crate::math::*;
use serde::{Deserialize, Serialize};

use crate::module::fractal::{atlas::PADDING, TEXTURE_SIZE};

#[derive(Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, Debug)]
pub struct TilePos {
    pub z: i8,
    pub x: u64,
    pub y: u64,
}

impl TilePos {
    pub fn root() -> TilePos {
        TilePos { x: 0, y: 0, z: 0 }
    }

    pub fn from_f64(p: Vector2<f64>, z: i8) -> TilePos {
        let s = (1_u64 << z) as f64;
        TilePos {
            x: (p.x * s) as u64,
            y: (p.y * s) as u64,
            z,
        }
    }

    pub fn parent(&mut self) {
        if self.z > 0 {
            self.x /= 2;
            self.y /= 2;
            self.z -= 1;
        }
    }

    pub fn child(&mut self, dx: u64, dy: u64) {
        assert!(self.z < 64);
        self.x = self.x * 2 + dx;
        self.y = self.y * 2 + dy;
        self.z += 1;
    }

    pub fn max_size(&self) -> u64 {
        1_u64 << self.z
    }

    fn tile_scale(&self) -> f64 {
        1.0 / self.max_size() as f64
    }

    pub fn to_f64(&self) -> [f64; 3] {
        let s = self.tile_scale();
        [self.x as f64 * s, self.y as f64 * s, s]
    }

    pub fn to_f64_with_padding(&self) -> [f64; 3] {
        let s = self.tile_scale();
        let pad = s * (PADDING as f64 / TEXTURE_SIZE as f64);
        [
            self.x as f64 * s - pad,
            self.y as f64 * s - pad,
            s + pad * 2.0,
        ]
    }
}
