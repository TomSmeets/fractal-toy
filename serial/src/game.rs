use sdl2::event::{Event, WindowEvent};

use crate::fractal::*;
use crate::input::*;
use crate::math::*;
use crate::sdl::*;
use crate::window::Window;

// TODO: implemnt save and load, this will handle some types that dont work with reload.
// For example the btreemap
pub struct State {
    sdl: Sdl,
    input: Input,
    fractal: Fractal,
    window: Window,
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
        let window = Window::new(&sdl);

        // TODO: get window size
        State {
            sdl,
            input: Input::new(),
            window,
            fractal: Fractal::new(),
        }
    }

    pub fn update(&mut self) -> bool {
        let dt = 1.0 / 60.0;

        self.sdl.update();
        self.window.update(&self.sdl);
        self.input.update(&self.sdl);

        let down = self.input.is_down(InputAction::A);
        self.fractal
            .update(dt, down, &mut self.sdl, &self.window, &self.input);

        if self.input.is_down(InputAction::F1) {
            println!("---- INFO ----");
            self.fractal.info(&self.input, &self.window);
        }

        self.input.is_down(InputAction::Quit)
    }
}
