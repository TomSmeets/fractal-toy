use super::rect::Rect;
use crate::math::V2i;
use crate::sdl::Sdl;
use sdl2::pixels::Color;
use serde::{Deserialize, Serialize};

use super::Collection;

#[derive(Serialize, Deserialize)]
pub struct Element {
    size: V2i,
}

impl Element {
    pub fn draw(&self, sdl: &mut Sdl, pos: V2i) {
        let rect = Rect {
            pos,
            size: self.size,
        };
        draw_rect(sdl, rect, [255, 255, 255]);
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Window {
    pub z_index: i32,
    pub rect: Rect,
    pub color: [u8; 3],

    pub items: Collection<Element>,
}

fn draw_rect(sdl: &mut Sdl, r: Rect, color: [u8; 3]) {
    let r = r.into_sdl();
    let mut r2 = r;
    r2.x -= 2;
    r2.y -= 2;
    r2.w += 4;
    r2.h += 4;

    sdl.canvas.set_draw_color(Color::RGB(0, 0, 0));
    sdl.canvas.fill_rect(r2).unwrap();

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

    pub fn begin(&mut self) {
        self.items.begin();
    }

    pub fn draw(&self, title: &str, sdl: &mut Sdl) {
        draw_rect(sdl, self.body_rect(), self.color);
        draw_rect(sdl, self.header_rect(), [64, 64, 128]);

        sdl.canvas.set_clip_rect(self.header_rect().into_sdl());
        {
            let (mut rect, texture) = sdl.make_text(title, 20.0);
            let hdr = self.header_rect();
            rect.x = hdr.pos.x + hdr.size.x / 2 - rect.w / 2;
            rect.y = hdr.pos.y + hdr.size.y / 2 - rect.h / 2;
            sdl.draw_rgba(rect, &texture);
        }

        let mut body = self.body_rect();
        sdl.canvas.set_clip_rect(body.into_sdl());
        draw_rect(sdl, self.resize_handle_rect(), [64, 64, 128]);

        for (_id, e) in self.items.iter() {
            e.draw(sdl, body.pos);
            let s = e.size.y + 10;
            body.pos.y += s;
            body.size.y -= s;
        }
        sdl.canvas.set_clip_rect(None);
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

    pub fn item(&mut self, id: &str) -> &mut Element {
        self.items.item(id, || Element {
            size: V2i::new(20, 20),
        })
    }
}
