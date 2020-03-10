use sdl2::event::{Event, WindowEvent};

use crate::fractal::*;
use crate::input::*;
use crate::math::*;
use crate::sdl::*;

pub type WindowSize = Vector2<u32>;

// TODO: implemnt save and load, this will handle some types that dont work with reload.
// For example the btreemap
pub struct State {
    sdl: Sdl,
    input: Input,
    fractal: Fractal,
    window_size: WindowSize,
}

impl Default for State {
    fn default() -> State {
        State::new()
    }
}

impl State {
    pub fn unload(&mut self) {}

    pub fn reload(&mut self) {}

    pub fn new() -> State {
        let sdl = Sdl::new();

        // TODO: get window size
        State {
            sdl,
            input: Input::new(),
            window_size: Vector2::new(800, 600),
            fractal: Fractal::new(),
        }
    }

    pub fn update(&mut self) -> bool {
        let dt = 1.0 / 60.0;

        let events: Vec<_> = self.sdl.event.poll_iter().collect();

        self.input.begin();
        self.input.handle_sdl(&events);
        self.input.end();

        for event in events {
            match event {
                Event::Window {
                    win_event: WindowEvent::Resized(x, y),
                    ..
                } => {
                    self.window_size.x = (x as u32).max(1);
                    self.window_size.y = (y as u32).max(1);
                },
                _ => {},
            }
        }

        let down = self.input.is_down(InputAction::A);
        self.fractal
            .update(dt, down, &mut self.sdl, self.window_size, &self.input);

        if self.input.is_down(InputAction::F1) {
            println!("---- INFO ----");
            self.fractal.info(&self.input, self.window_size);
        }

        self.input.is_down(InputAction::Quit)
    }
}
