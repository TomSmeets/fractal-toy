use crate::math::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Rect {
    pub pos: V2i,
    pub size: V2i,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect {
            pos: V2i::new(x, y),
            size: V2i::new(w, h),
        }
    }

    pub fn is_inside(&self, p: V2i) -> bool {
        p.x >= self.pos.x
            && p.y >= self.pos.y
            && p.x < self.pos.x + self.size.x
            && p.y < self.pos.y + self.size.y
    }

    #[cfg(feature = "sdl2")]
    pub fn to_sdl(&self) -> sdl2::rect::Rect {
        sdl2::rect::Rect::new(
            self.pos.x,
            self.pos.y,
            self.size.x as u32,
            self.size.y as u32,
        )
    }
}

impl Default for Rect {
    fn default() -> Self {
        Rect {
            pos: V2i::zero(),
            size: V2i::zero(),
        }
    }
}
