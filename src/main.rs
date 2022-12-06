// Sorry, but these warnings are very annoying
#![allow(dead_code)]
#![allow(unused_variables)]

use crate::fractal::Fractal;
use crate::state::State;
use crate::update_loop::Loop;

mod asset_loader;
mod builder;
mod debug;
mod fractal;
mod glyph_cache;
mod gpu;
mod image;
mod pack;
mod shelf_pack;
mod state;
mod tilemap;
mod ui;
mod update_loop;
mod util;
mod viewport;

pub fn main() {
    let update_loop = Loop::new("Fractal Toy!");

    let mut state = State::init(&update_loop.window);
    let mut fractal = Fractal::init(&mut state);
    update_loop.run(move |window, input| {
        fractal.update(&mut state, window, input);
        state.update(window, input);
    });
}
