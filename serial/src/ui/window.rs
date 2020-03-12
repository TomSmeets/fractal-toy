use super::rect::Rect;
use crate::math::V2i;
use crate::sdl::Sdl;
use sdl2::pixels::Color;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Window {
    pub z_index: i32,
    // TODO: restrict acess, this filed will always read as false to the user
    pub visible: bool,
    pub rect: Rect,
    pub color: [u8; 3],
}

fn draw_rect(sdl: &mut Sdl, r: Rect, color: [u8; 3]) {
    let mut r = r.into_sdl();
    {
        sdl.canvas.set_draw_color(Color::RGB(0, 0, 0));
        sdl.canvas.fill_rect(r).unwrap();
        r.x += 1;
        r.y += 1;
        r.w -= 2;
        r.h -= 2;
    }
    sdl.canvas
        .set_draw_color(Color::RGB(color[0], color[1], color[2]));
    sdl.canvas.fill_rect(r).unwrap();
}

impl Window {
    pub fn new() -> Self {
        Window {
            rect: Rect {
                pos: V2i::new(0, 0),
                size: V2i::new(80, 80),
            },
            color: [128, 128, 128],
            ..Self::default()
        }
    }

    pub fn draw(&self, sdl: &mut Sdl) {
        draw_rect(sdl, self.body_rect(), self.color);
        draw_rect(sdl, self.header_rect(), [64, 64, 128]);
        draw_rect(sdl, self.resize_handle_rect(), [64, 64, 128]);
    }

    pub fn header_rect(&self) -> Rect {
        let mut r = self.rect;
        r.size.y = 20;
        r
    }

    pub fn body_rect(&self) -> Rect {
        let mut r = self.rect;
        r.pos.y += 20;
        r.size.y -= 20;
        r
    }

    pub fn resize_handle_rect(&self) -> Rect {
        let mut r = Rect::default();
        r.size.x = 20;
        r.size.y = 20;
        r.pos = self.rect.pos + self.rect.size - r.size;
        r
    }

    pub fn is_inside(&self, p: V2i) -> bool {
        self.rect.is_inside(p)
    }
}
