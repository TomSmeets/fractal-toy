use crate::math::*;
use serde::{Deserialize, Serialize};

use crate::module::fractal::{atlas::PADDING, TEXTURE_SIZE};

#[derive(Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, Debug)]
pub struct TilePos {
    pub z: u8,
    pub x: i64,
    pub y: i64,
}

#[rustfmt::skip]
#[test]
fn test_fromf64() {
    assert_eq!(TilePos::from_f64(V2::new(0.0,  0.0),  0), TilePos { x: 0, y: 0, z: 0 });
    assert_eq!(TilePos::from_f64(V2::new(0.5,  0.5),  0), TilePos { x: 0, y: 0, z: 0 });
    assert_eq!(TilePos::from_f64(V2::new(0.9,  0.9),  0), TilePos { x: 0, y: 0, z: 0 });
    assert_eq!(TilePos::from_f64(V2::new(1.01, 1.01), 0), TilePos { x: 1, y: 1, z: 0 });

    assert_eq!(TilePos::from_f64(V2::new(0.0, 0.0), 1), TilePos { x: 0, y: 0, z: 1 });
    assert_eq!(TilePos::from_f64(V2::new(0.4, 0.4), 1), TilePos { x: 0, y: 0, z: 1 });
    assert_eq!(TilePos::from_f64(V2::new(0.5, 0.4), 1), TilePos { x: 1, y: 0, z: 1 });
    assert_eq!(TilePos::from_f64(V2::new(0.4, 0.5), 1), TilePos { x: 0, y: 1, z: 1 });

    assert_eq!(TilePos::from_f64(V2::new(0.0, 0.0), 16), TilePos { x: 0, y: 0, z: 16 });
}

impl TilePos {
    pub fn root() -> TilePos {
        TilePos { x: 0, y: 0, z: 0 }
    }

    pub fn from_f64(p: V2, z: u8) -> TilePos {
        let s = (1_u64 << z) as f64;
        TilePos {
            x: (p.x * s).floor() as i64,
            y: (p.y * s).floor() as i64,
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

    pub fn child(&mut self, dx: i64, dy: i64) {
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
