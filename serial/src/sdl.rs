use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;

pub struct Sdl {
    pub ctx: sdl2::Sdl,
    pub video: sdl2::VideoSubsystem,
    pub event: sdl2::EventPump,
    pub canvas: Canvas<Window>,
    pub events: Vec<sdl2::event::Event>,
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
            events: Vec::new(),
        }
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
