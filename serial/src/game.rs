use crate::fractal::*;
use crate::input::*;
use crate::math::*;
use crate::sdl::*;
use crate::window::Window;

pub struct Time {
    pub dt: f32,
    pub dt_inv: f32,
    pub iteration: i32,
    pub time: f32,
}

impl Time {
    pub fn new(dt: f32) -> Self {
        Time {
            dt,
            dt_inv: 1.0 / dt,
            iteration: 0,
            time: 0.0,
        }
    }

    pub fn update(&mut self) {
        self.iteration += 1;
        self.time += self.dt;
    }
}

// TODO: implemnt save and load, this will handle some types that dont work with reload.
// For example the btreemap
pub struct State {
    time: Time,
    sdl: Sdl,
    window: Window,
    input: Input,
    fractal: Fractal,
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
            time: Time::new(1.0 / 60.0),
            sdl,
            input: Input::new(),
            window,
            fractal: Fractal::new(),
        }
    }

    pub fn update(&mut self) -> bool {
        self.time.update();
        self.sdl.update();
        self.window.update(&self.sdl);
        self.input.update(&self.sdl);

        self.fractal
            .update(&self.time, &mut self.sdl, &self.window, &self.input);

        if self.input.is_down(InputAction::F1) {
            println!("---- INFO ----");
            self.fractal.info(&self.input, &self.window);
        }

        self.input.is_down(InputAction::Quit)
    }
}
