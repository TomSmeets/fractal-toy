// TODO: Do something with padding,
// either make it always standard or only atlas specific
// preferably make padding optional, but it might be to hard
use crate::fractal::PADDING;
use crate::fractal::TEXTURE_SIZE;
use crate::math::V2;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Debug)]
pub struct TilePos {
    z: u8,
    x: i64,
    y: i64,
}

#[rustfmt::skip]
#[test]
fn test_fromf64() {
    assert_eq!(TilePos::from_real(0.0,  0.0,  0), TilePos { x: 0, y: 0, z: 0 });
    assert_eq!(TilePos::from_real(0.5,  0.5,  0), TilePos { x: 0, y: 0, z: 0 });
    assert_eq!(TilePos::from_real(0.9,  0.9,  0), TilePos { x: 0, y: 0, z: 0 });
    assert_eq!(TilePos::from_real(1.01, 1.01, 0), TilePos { x: 1, y: 1, z: 0 });
    assert_eq!(TilePos::from_real(0.0, 0.0, 1), TilePos { x: 0, y: 0, z: 1 });
    assert_eq!(TilePos::from_real(0.4, 0.4, 1), TilePos { x: 0, y: 0, z: 1 });
    assert_eq!(TilePos::from_real(0.5, 0.4, 1), TilePos { x: 1, y: 0, z: 1 });
    assert_eq!(TilePos::from_real(0.4, 0.5, 1), TilePos { x: 0, y: 1, z: 1 });
    assert_eq!(TilePos::from_real(0.0, 0.0, 16), TilePos { x: 0, y: 0, z: 16 });
}

impl TilePos {
    pub fn root() -> TilePos {
        TilePos { x: 0, y: 0, z: 0 }
    }

    pub fn from_real(x: f64, y: f64, z: u8) -> TilePos {
        let s = (1_u64 << z) as f64;
        TilePos {
            x: (x * s).floor() as i64,
            y: (y * s).floor() as i64,
            z,
        }
    }

    // Iterate over tiles between 'min' and 'max'
    pub fn between(min: V2, max: V2, z: u8, pad: i64) -> impl Iterator<Item = TilePos> {
        let min = TilePos::from_real(min.x, min.y, z);
        let max = TilePos::from_real(max.x, max.y, z);
        let rx = (min.x - pad)..(max.x + pad + 1);
        let ry = (min.y - pad)..(max.y + pad + 1);
        rx.flat_map(move |x| ry.clone().map(move |y| TilePos { x, y, z }))
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
