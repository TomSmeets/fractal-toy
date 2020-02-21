pub mod state;
pub mod game;

use crate::game::*;

#[no_mangle]
pub extern fn prog_update(state: *mut State) -> bool {
    let s = unsafe { state.as_mut().unwrap() };
    s.update()
}

#[no_mangle]
pub extern fn prog_init() -> *mut State {
    return Box::into_raw(Box::new(State::new()));
}
