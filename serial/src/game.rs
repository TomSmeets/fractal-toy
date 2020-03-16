use crate::fractal::Fractal;
use crate::input::{Input, InputAction};
use crate::math::*;
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

            let _ = ui.button("hi");
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

            ui.window("hi", |ui| {
                if ui.button("Foo") {
                    println!("Foooooooo!");
                }

                if ui.button("Bar") {
                    println!("Bar");
                }
            });
        }

        input.is_down(InputAction::Quit)
    }
}
