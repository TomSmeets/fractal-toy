use sdl2::pixels::Color;
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

static FONT_DATA: &[u8] = include_bytes!(concat!(env!("FONT_DEJAVU"), "/share/fonts/truetype/DejaVuSansMono.ttf"));

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

    pub fn update(&mut self) {
        let l = self.font.layout(
            "Hello World",
            rusttype::Scale::uniform(100.0),
            rusttype::point(20.0, 20.0),
        );

        self.canvas.set_draw_color(Color::RGB(255, 255, 255));
        let mut img: Vec<(i32, i32, f32)> = Vec::new();
        for i in l {
            if let Some(p) = i.pixel_bounding_box() {
                i.draw(|x, y, c| {
                    img.push((p.min.x as i32 + x as i32, p.min.y as i32 + y as i32, c))
                });
            }
        }

        for (x, y, c) in img {
            let c = (c * 255.) as u8;
            if c > 0 {
                self.canvas.set_draw_color(Color::RGBA(c, c, c, c));
                self.canvas
                    .fill_rect(sdl2::rect::Rect::new(x, y + 100, 1, 1))
                    .unwrap();
            }
        }

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
