use crate::math::*;
use sdl2::{
    pixels::{Color, PixelFormatEnum},
    rect::Rect,
    render::{BlendMode, Canvas},
    video::Window,
};

pub struct Sdl {
    pub events: Vec<sdl2::event::Event>,
}

pub mod ctx {
    use rusttype::Font;
    use sdl2::{render::Canvas, video::Window};

    pub type Texture = sdl2::render::Texture<'static>;
    pub type TextureCreator = sdl2::render::TextureCreator<sdl2::video::WindowContext>;

    static FONT_DATA: &[u8] = include_bytes!(concat!(env!("FONT_DEJAVU"), "/DejaVuSans.ttf"));
    static mut SDL: Option<SdlContext> = None;

    pub struct SdlContext {
        pub ctx: sdl2::Sdl,
        pub video: sdl2::VideoSubsystem,
        pub event: sdl2::EventPump,
        pub font: Font<'static>,
        pub canvas: Option<Canvas<Window>>,
        pub texture_creator: Option<TextureCreator>,
    }

    impl SdlContext {
        pub fn new() -> Self {
            let ctx = sdl2::init().unwrap();
            let video = ctx.video().unwrap();
            let event = ctx.event_pump().unwrap();
            let font: Font<'static> = Font::from_bytes(FONT_DATA).unwrap();
            SdlContext {
                ctx,
                video,
                event,
                font,
                canvas: None,
                texture_creator: None,
            }
        }
    }

    impl Drop for SdlContext {
        fn drop(&mut self) {
            self.texture_creator = None;
            self.canvas = None;
        }
    }

    pub fn init() {
        unsafe {
            assert!(SDL.is_none());
            SDL = Some(SdlContext::new());
        }
    }

    pub fn get() -> &'static mut SdlContext {
        unsafe {
            assert!(SDL.is_some());
            SDL.as_mut().unwrap()
        }
    }

    pub fn exit() {
        unsafe {
            assert!(SDL.is_some());
            SDL = None;
        }
    }
}

impl Sdl {
    pub fn new() -> Self {
        let ctx = ctx::get();
        ctx.canvas = None;

        let window = ctx
            .video
            .window("rust-sdl2 demo", 800, 600)
            .resizable()
            .opengl()
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().present_vsync().build().unwrap();
        canvas.set_blend_mode(BlendMode::Blend);
        ctx.texture_creator = Some(canvas.texture_creator());
        ctx.canvas = Some(canvas);

        unsafe {
            sdl2::sys::SDL_SetHint(
                sdl2::sys::SDL_HINT_RENDER_SCALE_QUALITY.as_ptr() as *const i8,
                (b"1").as_ptr() as *const i8,
            );
        }

        Sdl { events: Vec::new() }
    }

    pub fn canvas(&self) -> &Canvas<Window> {
        ctx::get().canvas.as_ref().unwrap()
    }

    pub fn canvas_mut(&mut self) -> &mut Canvas<Window> {
        ctx::get().canvas.as_mut().unwrap()
    }

    pub fn texture_creator(&self) -> &'static ctx::TextureCreator {
        ctx::get().texture_creator.as_ref().unwrap()
    }

    pub fn draw_text(&mut self, text: &str, pos: V2i) {
        let (mut r, t) = self.make_text(text, 40.0);
        r.x += pos.x;
        r.y += pos.y;
        self.draw_rgba(r, &t);
    }

    pub fn draw_rgba(&mut self, r: Rect, p: &[u8]) {
        let mut texture: ctx::Texture = self
            .texture_creator()
            .create_texture_static(PixelFormatEnum::RGBA8888, r.w as u32, r.h as u32)
            .unwrap();
        texture.set_blend_mode(BlendMode::Blend);
        texture.update(None, &p, 4 * r.w as usize).unwrap();
        self.canvas_mut().copy(&texture, None, Some(r)).unwrap();
    }

    pub fn make_text(&self, text: &str, size: f32) -> (Rect, Vec<u8>) {
        let ctx = ctx::get();
        let l = ctx.font.layout(
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
        self.canvas_mut().present();
        self.events = ctx::get().event.poll_iter().collect();
        self.canvas_mut().set_draw_color(Color::RGB(255, 0, 255));
        self.canvas_mut().clear();
    }
}

impl Default for Sdl {
    fn default() -> Self {
        Self::new()
    }
}
