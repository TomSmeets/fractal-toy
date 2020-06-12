pub use cgmath::*;

mod rect;
pub use self::rect::Rect;

pub type Real = f64;
pub type V2 = Vector2<f64>;
pub type V2i = Vector2<i32>;

pub fn to_v2i(v: V2) -> V2i {
    V2i::new(v.x as i32, v.y as i32)
}
pub fn to_v2<T: Into<f64>>(v: Vector2<T>) -> V2 {
    v.map(|x| x.into())
}

pub struct FRect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl FRect {
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        FRect { x, y, w, h }
    }

    /// Grow this rectangle on all sides by 's'
    /// NOTE: total size will increase by 2*n*{w, h}!
    pub fn grow_relative(self, s: f64) -> Self {
        let sx = s * self.w;
        let sy = s * self.h;
        Self {
            x: self.x - sx,
            y: self.y - sy,
            w: self.w + sx + sx,
            h: self.h + sy + sy,
        }
    }

    /// The corner where x and y are smallest
    pub fn corner_min(&self) -> V2 {
        V2::new(self.x, self.y)
    }

    /// The corner where x and y are biggest
    pub fn corner_max(&self) -> V2 {
        V2::new(self.x + self.w, self.y + self.h)
    }

    /// The center of this tile
    pub fn center(&self) -> V2 {
        V2::new(self.x + self.w * 0.5, self.y + self.h * 0.5)
    }
}
