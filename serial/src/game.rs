use crate::{
    math::*,
    module::{input::InputAction, Fractal, Input, Sdl, Time, Window, UI},
};
use serde::{Deserialize, Serialize};

// NOTE: gpu contains a memory allocator, that is why drop should work

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
        let time = Time::new(1.0 / 60.0);
        let input = Input::new();
        let ui = UI::new();
        let fractal = Fractal::new();

        // TODO: get window size
        State {
            time,
            sdl,
            window,
            input,
            ui,
            fractal,
        }
    }

    pub fn update(&mut self) -> bool {
        let State {
            time,
            sdl,
            window,
            input,
            ui,
            fractal,
        } = self;

        time.update();
        sdl.update();
        window.update(sdl);
        input.update(sdl);

        fractal.update(time, sdl, window, input);

        if false {
            ui.update(
                sdl,
                input,
                V2i::new(window.size.x as i32, window.size.y as i32),
            );

            // let _ = ui.button("hi");
            ui.window("win", |ui| {
                if ui.button("Hello?") {
                    println!("Hello world!");
                }

                if ui.button("World?") {
                    println!("world!");
                }

                let _ = ui.button("a");
                let _ = ui.button("b");
                let _ = ui.button("c");
                let _ = ui.button("d");
            });

            // ui.window("hi", |ui| {
            //     if ui.button("Foo") {
            //         println!("Foooooooo!");
            //     }

            //     if ui.button("Bar") {
            //         println!("Bar");
            //     }
            // });
        }

        input.button(InputAction::Quit).is_down
    }
}
