use crate::module::{input::InputAction, Fractal, Input, Sdl, Time, Window};
use serde::{Deserialize, Serialize};

use crate::math::*;
use crate::module::atlas::Atlas;
use crate::module::atlas::AtlasRegion;
use crate::module::atlas::AtlasTextureCreator;

#[derive(Serialize, Deserialize)]
pub struct State {
    time: Time,
    #[serde(skip)]
    sdl: Sdl,
    window: Window,
    #[serde(skip)]
    pub input: Input,

    fractal: Fractal<AtlasRegion>,

    #[serde(skip)]
    atlas: Atlas,
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
            atlas: Atlas::new(),
        }
    }

    pub fn update(&mut self) -> bool {
        let State {
            time,
            sdl,
            window,
            input,
            fractal,
            atlas,
        } = self;

        time.update();
        sdl.update();
        window.update(sdl);
        input.update(sdl);

        let mut gen = AtlasTextureCreator { sdl, atlas };
        fractal.update(&mut gen, time, window, input);

        if fractal.debug {
            // Show atlas
            // TODO: show in ui window
            let w = window.size.x as i32 / atlas.texture.len().max(4) as i32;
            for (i, t) in atlas.texture.iter().enumerate() {
                sdl.canvas_copy(t, None, Some(Rect::new(i as i32 * w, 0, w, w).to_sdl()));
            }
        }

        input.button(InputAction::Quit).is_down
    }
}
