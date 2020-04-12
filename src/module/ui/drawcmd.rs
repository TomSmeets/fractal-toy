use crate::math::Rect;

pub enum DrawCmd {
    Rect(Rect, [f32; 4]),
    Text(Rect, String),
    Clip(Option<Rect>),
}
