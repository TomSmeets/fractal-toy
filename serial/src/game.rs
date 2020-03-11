use crate::fractal::Fractal;
use crate::input::{Input, InputAction};
use crate::sdl::Sdl;
use crate::ui::UI;
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
    ui: UI,
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
            window,
            input: Input::new(),
            ui: UI::new(),
            fractal: Fractal::new(),
        }
    }

    pub fn update(&mut self) -> bool {
        self.time.update();
        self.sdl.update();
        self.window.update(&self.sdl);
        self.input.update(&self.sdl);
        self.ui.update();

        self.fractal
            .update(&self.time, &mut self.sdl, &self.window, &self.input);

        self.input.is_down(InputAction::Quit)
    }
}
