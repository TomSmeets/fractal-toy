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
