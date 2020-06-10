use serde::{Deserialize, Serialize};

/// Tile position representation
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

    fn max_size(&self) -> u64 {
        1_u64 << self.z
    }

    /// the size of this tile in both x and y (all tiles are square)
    pub fn tile_scale(&self) -> f64 {
        1.0 / self.max_size() as f64
    }

    /// The corner where x and y are smallest
    pub fn pos_center(&self) -> [f64; 2] {
        let s = self.tile_scale();
        [self.x as f64 * s + s * 0.5, self.y as f64 * s + s * 0.5]
    }

    /// The corner where x and y are smallest
    pub fn pos_min(&self) -> [f64; 2] {
        let s = self.tile_scale();
        [self.x as f64 * s, self.y as f64 * s]
    }

    /// The corner where x and y are biggest
    pub fn pos_max(&self) -> [f64; 2] {
        let s = self.tile_scale();
        [(self.x + 1) as f64 * s, (self.y + 1) as f64 * s]
    }

    /// [ min_x, min_y, max_x, max_y ]
    pub fn rect(&self) -> [f64; 4] {
        let s = self.tile_scale();
        let x = self.x as f64 * s;
        let y = self.y as f64 * s;
        [x, y, x + s, y + s]
    }

    // pub fn to_f64_with_padding(&self) -> [f64; 3] {
    //     let s = self.tile_scale();
    //     let pad = s * (PADDING as f64 / TEXTURE_SIZE as f64);
    //     [
    //         self.x as f64 * s - pad,
    //         self.y as f64 * s - pad,
    //         s + pad * 2.0,
    //     ]
    // }
}

#[rustfmt::skip]
#[test]
fn test_root() {
    assert_eq!(TilePos::root().pos_min(),    [0.0, 0.0]);
    assert_eq!(TilePos::root().pos_center(), [0.5, 0.5]);
    assert_eq!(TilePos::root().pos_max(),    [1.0, 1.0]);
    assert_eq!(TilePos::root().tile_scale(), 1.0);
    assert_eq!(TilePos::root(), TilePos::at(0.0, 0.0, 0));
    assert_eq!(TilePos::root(), TilePos::at(0.4, 0.4, 0));
    assert_eq!(TilePos::root(), TilePos::at(0.9, 0.9, 0));
}

#[rustfmt::skip]
#[test]
fn test_fromf64() {
    assert_eq!(TilePos::at(0.0,  0.0,  0), TilePos { x: 0, y: 0, z: 0 });
    assert_eq!(TilePos::at(0.5,  0.5,  0), TilePos { x: 0, y: 0, z: 0 });
    assert_eq!(TilePos::at(0.9,  0.9,  0), TilePos { x: 0, y: 0, z: 0 });
    assert_eq!(TilePos::at(1.01, 1.01, 0), TilePos { x: 1, y: 1, z: 0 });
    assert_eq!(TilePos::at(0.0,  0.0,  1), TilePos { x: 0, y: 0, z: 1 });
    assert_eq!(TilePos::at(0.4,  0.4,  1), TilePos { x: 0, y: 0, z: 1 });
    assert_eq!(TilePos::at(0.5,  0.4,  1), TilePos { x: 1, y: 0, z: 1 });
    assert_eq!(TilePos::at(0.4,  0.5,  1), TilePos { x: 0, y: 1, z: 1 });
    assert_eq!(TilePos::at(0.0,  0.0, 16), TilePos { x: 0, y: 0, z: 16 });
}
