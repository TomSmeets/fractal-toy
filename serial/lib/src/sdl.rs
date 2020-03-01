use sdl2::render::Canvas;
use sdl2::video::Window;

pub struct Sdl {
    pub ctx: sdl2::Sdl,
    pub video: sdl2::VideoSubsystem,
    pub event: sdl2::EventPump,
    pub canvas: Canvas<Window>,
}

impl Sdl {
    pub fn new() -> Self {
        let ctx = sdl2::init().unwrap();
        let video = ctx.video().unwrap();

        let window = video.window("rust-sdl2 demo", 800, 600).position_centered().build().unwrap();

        let event = ctx.event_pump().unwrap();
        let canvas = window.into_canvas().present_vsync().build().unwrap();

        Sdl { ctx, video, event, canvas }
    }
}

impl Default for Sdl {
    fn default() -> Self {
        Self::new()
    }
}
