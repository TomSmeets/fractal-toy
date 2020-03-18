extern crate sdl2;

mod game;
use game::Game;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use std::process;

struct State {
    #[allow(dead_code)]
    ctx: sdl2::Sdl,
    #[allow(dead_code)]
    video_ctx: sdl2::VideoSubsystem,
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    events: sdl2::EventPump,
    rect: Rect,
}

impl Game for State {
    fn init() -> Self {
        let ctx = sdl2::init().unwrap();
        let video_ctx = ctx.video().unwrap();

        let window = match video_ctx
            .window("rust_to_js", 640, 480)
            .position_centered()
            .opengl()
            .build()
        {
            Ok(window) => window,
            Err(err) => panic!("failed to create window: {}", err),
        };

        let rect = Rect::new(10, 10, 10, 10);
        let events = ctx.event_pump().unwrap();

        let canvas = window.into_canvas().present_vsync().build().unwrap();

        Self {
            ctx,
            events,
            canvas,
            video_ctx,
            rect,
        }
    }

    fn update(&mut self) {
        self.canvas
            .set_draw_color(sdl2::pixels::Color::RGB(255, 0, 255));
        self.canvas.clear();
        self.canvas.present();
        for event in self.events.poll_iter() {
            println!("event {:#?}", event);
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    process::exit(1);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    println!("rect: {:#?}", self.rect);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    self.rect.x -= 10;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    self.rect.x += 10;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    self.rect.y -= 10;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    self.rect.y += 10;
                }
                _ => {}
            }
        }
    }
}

#[cfg(target_os = "emscripten")]
mod emscripten;

#[cfg(target_os = "emscripten")]
use emscripten::run;

#[cfg(not(target_os = "emscripten"))]
fn run() {
    let mut s = State::init();

    loop {
        s.update()
    }
}

fn main() {
    println!("Hell oworld!");
    run();
}
