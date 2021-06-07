use super::Square;
use crate::util::V2;

#[derive(Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Debug)]
pub struct TilePos {
    pub z: u8,
    pub x: i64,
    pub y: i64,
}

impl TilePos {
    /// The root tile is the tile from 0,0 to 1,1
    pub fn root() -> TilePos {
        TilePos { x: 0, y: 0, z: 0 }
    }

    pub fn parent(&self) -> Option<TilePos> {
        if self.z == 0 {
            return None;
        }

        Some(TilePos { x: self.x / 2, y: self.y / 2, z: self.z - 1})
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
    pub fn between(min: V2, max: V2, z: u8, pad: i64, dst: &mut Vec<TilePos>) {
        let mut min = TilePos::at(min.x, min.y, z);
        let mut max = TilePos::at(max.x, max.y, z);

        // add padding
        min.x -= pad;
        min.y -= pad;
        max.x += pad;
        max.y += pad;

        let cx = min.x / 2 + max.x / 2;
        let cy = min.y / 2 + max.y / 2;

        let start = dst.len();
        dst.reserve(((max.x - min.x + 1)*(max.y - min.y + 1)) as usize);
        for y in min.y..max.y+1 {
            for x in min.x..max.x+1 {
                dst.push(TilePos { x, y, z });
            }
        }

        // sort center tiles first
        dst[start..].sort_by_key(|p| {
            let dx = p.x - cx;
            let dy = p.y - cy;
            dx*dx + dy*dy
        });
    }

    /// the size of this tile in both x and y (all tiles are square)
    pub fn tile_scale(&self) -> f64 {
        1.0 / self.max_size() as f64
    }

    pub fn square(&self) -> Square {
        let s = self.tile_scale();
        let x = self.x as f64 * s;
        let y = self.y as f64 * s;
        Square::new(x, y, s)
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
