use super::rect::Rect;
use crate::math::V2i;
use crate::sdl::Sdl;
use sdl2::pixels::Color;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Window {
    pub z_index: i32,
    pub visible: bool,
    pub rect: Rect,
    pub color: [u8; 3],
}

impl Window {
    pub fn new() -> Self {
        Window {
            rect: Rect {
                pos: V2i::new(0, 0),
                size: V2i::new(80, 80),
            },
            color: [255, 255, 255],
            ..Self::default()
        }
    }

    pub fn draw(&self, sdl: &mut Sdl) {
        let mut r = self.rect.into_sdl();
        sdl.canvas.set_draw_color(Color::RGB(0, 0, 0));
        sdl.canvas.fill_rect(r).unwrap();

        r.x += 4;
        r.y += 4;
        r.w -= 8;
        r.h -= 8;

        sdl.canvas
            .set_draw_color(Color::RGB(self.color[0], self.color[1], self.color[2]));
        sdl.canvas.fill_rect(r).unwrap();
    }

    pub fn is_inside(&self, p: V2i) -> bool {
        self.rect.is_inside(p)
    }
}
