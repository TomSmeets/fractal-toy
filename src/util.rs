pub use cgmath::prelude::*;
use cgmath::Vector2;
use cgmath::Vector3;

pub type V2<S = f64> = Vector2<S>;
pub type V3<S = f64> = Vector3<S>;

pub struct Rect {
    min: V2,
    max: V2,
}

impl Rect {
    pub fn min_max(min: V2, max: V2) -> Self {
        Rect { min, max }
    }

    pub fn corner_size(corner: V2, size: V2) -> Self {
        Rect {
            min: corner,
            max: corner + size,
        }
    }

    pub fn center_size(center: V2, size: V2) -> Self {
        Rect {
            min: center - size * 0.5,
            max: center + size * 0.5,
        }
    }

    pub fn corner_min(&self) -> V2 {
        self.min
    }

    pub fn corner_max(&self) -> V2 {
        self.max
    }

    pub fn center(&self) -> V2 {
        (self.max + self.min) * 0.5
    }

    pub fn size(&self) -> V2 {
        self.max - self.min
    }

    pub fn contains(&self, v: V2) -> bool {
        v.x >= self.min.x && v.x < self.max.x &&
        v.y >= self.min.y && v.y < self.max.y
    }
}
