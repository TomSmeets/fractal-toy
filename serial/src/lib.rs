// TODO: Arbirtrary precision
// TODO: compute tiles in background

pub mod game;
pub mod math;
pub mod state;

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
