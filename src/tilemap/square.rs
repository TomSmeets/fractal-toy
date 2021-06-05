use super::TilePos;
use crate::util::V2;

/// A rectangle but all sides are equal
pub struct Square {
    pub x: f64,
    pub y: f64,
    pub w: f64,
}

impl Square {
    pub fn new(x: f64, y: f64, w: f64) -> Self {
        Square { x, y, w }
    }

    /// Grow this rectangle on all sides by 's'
    /// NOTE: total size will increase by 2*n*{w, h}!
    pub fn grow_relative(self, s: f64) -> Self {
        let sx = s * self.w;
        Self {
            x: self.x - sx,
            y: self.y - sx,
            w: self.w + sx + sx,
        }
    }

    /// The corner where x and y are smallest
    pub fn corner_min(&self) -> V2 {
        V2::new(self.x, self.y)
    }

    /// The corner where x and y are biggest
    pub fn corner_max(&self) -> V2 {
        V2::new(self.x + self.w, self.y + self.w)
    }

    pub fn size(&self) -> f64 {
        self.w
    }

    /// The center of this tile
    pub fn center(&self) -> V2 {
        V2::new(self.x + self.w * 0.5, self.y + self.w * 0.5)
    }
}

impl From<TilePos> for Square {
    fn from(p: TilePos) -> Self {
        p.square()
    }
}
