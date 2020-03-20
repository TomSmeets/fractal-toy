// TODO: Arbirtrary precision
// TODO: compute tiles in background

pub mod game;
pub mod input;
pub mod math;
pub mod sdl;
pub mod state;
pub mod ui;
pub mod window;

pub mod module;

use crate::game::*;

#[no_mangle]
pub unsafe extern "C" fn prog_update(state: *mut State) -> bool {
    (*state).update()
}

#[no_mangle]
pub unsafe extern "C" fn prog_init() -> *mut State {
    Box::into_raw(Box::new(State::new()))
}
