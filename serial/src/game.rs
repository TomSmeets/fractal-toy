use crate::module::{input::InputAction, Fractal, Input, Sdl, Time, Window};
use serde::{Deserialize, Serialize};

// TODO: implemnt save and load, this will handle some types that dont work with
// reload. For example the btreemap
#[derive(Serialize, Deserialize)]
pub struct State {
    time: Time,
    #[serde(skip)]
    sdl: Sdl,
    window: Window,
    #[serde(skip)]
    pub input: Input,
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
        let time = Time::new(1.0 / 60.0);
        let input = Input::new();
        let fractal = Fractal::new();

        // TODO: get window size
        State {
            time,
            sdl,
            window,
            input,
            fractal,
        }
    }

    pub fn update(&mut self) -> bool {
        let State {
            time,
            sdl,
            window,
            input,
            fractal,
        } = self;

        time.update();
        sdl.update();
        window.update(sdl);
        input.update(sdl);

        fractal.update(time, sdl, window, input);

        input.button(InputAction::Quit).is_down
    }
}
