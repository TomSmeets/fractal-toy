use crate::math::FRect;
use crate::math::V2;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Debug)]
pub struct TilePos {
    z: u8,
    x: i64,
    y: i64,
}

impl TilePos {
    /// The root tile is the tile from 0,0 to 1,1
    pub fn root() -> TilePos {
        TilePos { x: 0, y: 0, z: 0 }
    }

    /// Create a tile at these coordinates and at a spesific depth
    pub fn at(x: f64, y: f64, z: u8) -> TilePos {
        let s = (1_u64 << z) as f64;
        TilePos {
            x: (x * s).floor() as i64,
            y: (y * s).floor() as i64,
            z,
        }
    }

    // Iterate over tiles between 'min' and 'max'
    pub fn between(min: V2, max: V2, z: u8, pad: i64) -> impl Iterator<Item = TilePos> {
        let min = TilePos::at(min.x, min.y, z);
        let max = TilePos::at(max.x, max.y, z);
        let rx = (min.x - pad)..(max.x + pad + 1);
        let ry = (min.y - pad)..(max.y + pad + 1);
        rx.flat_map(move |x| ry.clone().map(move |y| TilePos { x, y, z }))
    }

    /// the size of this tile in both x and y (all tiles are square)
    pub fn tile_scale(&self) -> f64 {
        1.0 / self.max_size() as f64
    }

    pub fn rect(&self) -> FRect {
        let s = self.tile_scale();
        let x = self.x as f64 * s;
        let y = self.y as f64 * s;
        FRect::new(x, y, s, s)
    }

    pub fn to_f64(&self) -> [f64; 3] {
        let s = self.tile_scale();
        [self.x as f64 * s, self.y as f64 * s, s]
    }

    fn max_size(&self) -> u64 {
        1_u64 << self.z
    }
}

#[rustfmt::skip]
#[test]
fn test_fromf64() {
    assert_eq!(TilePos::at(0.0,  0.0,  0), TilePos { x: 0, y: 0, z: 0 });
    assert_eq!(TilePos::at(0.5,  0.5,  0), TilePos { x: 0, y: 0, z: 0 });
    assert_eq!(TilePos::at(0.9,  0.9,  0), TilePos { x: 0, y: 0, z: 0 });
    assert_eq!(TilePos::at(1.01, 1.01, 0), TilePos { x: 1, y: 1, z: 0 });
    assert_eq!(TilePos::at(0.0, 0.0, 1), TilePos { x: 0, y: 0, z: 1 });
    assert_eq!(TilePos::at(0.4, 0.4, 1), TilePos { x: 0, y: 0, z: 1 });
    assert_eq!(TilePos::at(0.5, 0.4, 1), TilePos { x: 1, y: 0, z: 1 });
    assert_eq!(TilePos::at(0.4, 0.5, 1), TilePos { x: 0, y: 1, z: 1 });
    assert_eq!(TilePos::at(0.0, 0.0, 16), TilePos { x: 0, y: 0, z: 16 });
}
