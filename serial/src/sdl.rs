use crate::math::*;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::Canvas;
use sdl2::video::Window;

use sdl2::rect::Rect;
use sdl2::render::Texture;

use rusttype::Font;

pub struct Sdl {
    pub ctx: sdl2::Sdl,
    pub video: sdl2::VideoSubsystem,
    pub event: sdl2::EventPump,
    pub canvas: Canvas<Window>,
    pub events: Vec<sdl2::event::Event>,

    pub font: Font<'static>,
}

static FONT_DATA: &[u8] = include_bytes!(concat!(
    env!("FONT_DEJAVU"),
    "/share/fonts/truetype/DejaVuSans.ttf"
));

impl Sdl {
    pub fn new() -> Self {
        let ctx = sdl2::init().unwrap();
        let video = ctx.video().unwrap();

        let window = video
            .window("rust-sdl2 demo", 800, 600)
            .position_centered()
            .build()
            .unwrap();

        let event = ctx.event_pump().unwrap();
        let canvas = window.into_canvas().present_vsync().build().unwrap();

        let font: Font<'static> = Font::from_bytes(FONT_DATA).unwrap();

        Sdl {
            ctx,
            video,
            event,
            canvas,
            events: Vec::new(),
            font,
        }
    }

    pub fn draw_text(&mut self, text: &str, pos: V2i) {
        let (mut r, t) = self.make_text(text, 40.0);
        r.x += pos.x;
        r.y += pos.y;
        self.draw_rgba(r, &t);
    }

    pub fn draw_rgba(&mut self, r: Rect, p: &[u8]) {
        let mut texture = self
            .canvas
            .create_texture_static(PixelFormatEnum::RGBA8888, r.w as u32, r.h as u32)
            .unwrap();
        texture.set_blend_mode(sdl2::render::BlendMode::Blend);
        texture.update(None, &p, 4 * r.w as usize).unwrap();
        self.canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
        self.canvas.copy(&texture, None, Some(r)).unwrap();

        unsafe {
            texture.destroy();
        }
    }

    pub fn make_text(&self, text: &str, size: f32) -> (Rect, Vec<u8>) {
        let l = self.font.layout(
            text,
            rusttype::Scale::uniform(size),
            rusttype::point(0.0, 0.0),
        );
        let glyphs: Vec<_> = l.collect();

        let boxes: Vec<_> = glyphs
            .iter()
            .filter_map(|g| g.pixel_bounding_box())
            .collect();

        let min = V2i {
            x: boxes.iter().map(|x| x.min.x).min().unwrap(),
            y: boxes.iter().map(|x| x.min.y).min().unwrap(),
        };

        let max = V2i {
            x: boxes.iter().map(|x| x.max.x).max().unwrap(),
            y: boxes.iter().map(|x| x.max.y).max().unwrap(),
        };

        let global_w = (max.x - min.x) as u32;
        let global_h = (max.y - min.y) as u32;

        let mut pixels = vec![0; global_w as usize * global_h as usize * 4];
        for i in glyphs {
            if let Some(p) = i.pixel_bounding_box() {
                i.draw(|x, y, c| {
                    let x = (p.min.x - min.x + x as i32) as usize;
                    let y = (p.min.y - min.y + y as i32) as usize;
                    let c = (c * 255.0) as u8;
                    pixels[y * global_w as usize * 4 + x * 4 + 0] = c;
                    pixels[y * global_w as usize * 4 + x * 4 + 1] = 255;
                    pixels[y * global_w as usize * 4 + x * 4 + 2] = 255;
                    pixels[y * global_w as usize * 4 + x * 4 + 3] = 255;
                });
            }
        }

        let r = sdl2::rect::Rect::new(min.x, min.y, global_w, global_h);
        (r, pixels)
    }

    pub fn update(&mut self) {
        self.canvas.present();
        self.events = self.event.poll_iter().collect();
        self.canvas.set_draw_color(Color::RGB(32, 32, 32));
        self.canvas.clear();
    }
}

impl Default for Sdl {
    fn default() -> Self {
        Self::new()
    }
}
