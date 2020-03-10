// TODO: Arbirtrary precision
// TODO: compute tiles in background

pub mod fractal;
pub mod game;
pub mod input;
pub mod math;
pub mod quadtree;
pub mod sdl;
pub mod state;
pub mod viewport;

use crate::game::*;

#[no_mangle]
pub extern "C" fn prog_update(state: *mut State) -> bool {
    unsafe { (*state).update() }
}

#[no_mangle]
pub extern "C" fn prog_init() -> *mut State {
    Box::into_raw(Box::new(State::new()))
}
