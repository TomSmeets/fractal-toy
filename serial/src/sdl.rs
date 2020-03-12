use crate::math::*;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::Canvas;
use sdl2::video::Window;

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
        let l = self.font.layout(
            text,
            rusttype::Scale::uniform(20.0),
            rusttype::point(0.0, 0.0),
        );
        self.canvas.set_draw_color(Color::RGB(255, 255, 255));
        self.canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
        let glyphs: Vec<_> = l.collect();
        for i in glyphs {
            if let Some(p) = i.pixel_bounding_box() {
                let w = (p.max.x - p.min.x) as usize;
                let h = (p.max.y - p.min.y) as usize;
                let mut pixels = vec![0; w * h * 4];

                i.draw(|x, y, c| {
                    let x = x as usize;
                    let y = y as usize;
                    let c = (c * 255.0) as u8;
                    pixels[y * w * 4 + x * 4 + 0] = c;
                    pixels[y * w * 4 + x * 4 + 1] = c;
                    pixels[y * w * 4 + x * 4 + 2] = c;
                    pixels[y * w * 4 + x * 4 + 3] = c;
                });

                let mut texture = self
                    .canvas
                    .create_texture_static(PixelFormatEnum::RGBA8888, w as u32, h as u32)
                    .unwrap();
                texture.set_blend_mode(sdl2::render::BlendMode::Blend);

                texture.update(None, &pixels, 4 * w).unwrap();

                let r = sdl2::rect::Rect::new(p.min.x + pos.x, p.min.y + pos.y, w as u32, h as u32);
                self.canvas.copy(&texture, None, Some(r)).unwrap();
            }
        }
    }

    pub fn update(&mut self) {
        self.draw_text("Hello World", V2i::new(0, 100));
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
