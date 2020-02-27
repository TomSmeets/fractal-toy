pub mod fractal;
pub mod game;
pub mod input;
pub mod math;
pub mod quadtree;
pub mod state;

use crate::game::*;

#[no_mangle]
pub extern "C" fn prog_update(state: &mut State) -> bool {
	state.update()
}

#[no_mangle]
pub extern "C" fn prog_init() -> *mut State {
	return Box::into_raw(Box::new(State::new()));
}
