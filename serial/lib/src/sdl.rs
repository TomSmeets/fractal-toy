use sdl2::event::*;
use sdl2::keyboard::Keycode;
use sdl2::pixels::*;
use sdl2::rect::*;
use sdl2::render::*;
use sdl2::video::*;

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

        let window = video
            .window("rust-sdl2 demo", 800, 600)
            .position_centered()
            .build()
            .unwrap();

        let event = ctx.event_pump().unwrap();
        let canvas = window.into_canvas().present_vsync().build().unwrap();

        Sdl {
            ctx,
            video,
            event,
            canvas,
        }
    }
}
