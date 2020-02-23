pub mod state;
pub mod input;
pub mod fractal;
pub mod game;

use crate::game::*;

#[no_mangle]
pub extern fn prog_update(state: &mut State) -> bool {
    state.update()
}

#[no_mangle]
pub extern fn prog_init() -> *mut State {
    return Box::into_raw(Box::new(State::new()));
}
