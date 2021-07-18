// Sorry, but these warnings are very annoying
#![allow(dead_code)]
#![allow(unused_variables)]

mod asset_loader;
mod builder;
mod debug;
mod glyph_cache;
mod gpu;
mod image;
mod pack;
mod shelf_pack;
mod tilemap;
mod ui;
mod update_loop;
mod util;
mod viewport;
mod fractal;

use crate::update_loop::Loop;
use crate::fractal::State;


pub fn main() {
    let update_loop = Loop::new("Fractal Toy!");

    let mut state = State::init(&update_loop.window);
    update_loop.run(move |window, input| state.update(window, input));
}
