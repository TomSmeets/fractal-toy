use std::cell::RefCell;
use std::os::raw::{c_float, c_int, c_void};
use std::ptr::null_mut;
use crate::game::Game;

#[allow(non_camel_case_types)]
type em_callback_func = unsafe extern "C" fn();

extern "C" {
    pub fn emscripten_set_main_loop(
        func: em_callback_func,
        fps: c_int,
        simulate_infinite_loop: c_int,
    );
    pub fn emscripten_cancel_main_loop();
    pub fn emscripten_get_now() -> c_float;
}

static mut STATE: Option<Box<dyn Game>> = None;

unsafe extern "C" fn prog_update() {
    match &mut STATE {
        Some(x) => {
            x.update();
        }
        None => {}
    }
}

pub fn run(g: Box<dyn Game>) {
    unsafe {
        STATE = Some(g);
        emscripten_set_main_loop(prog_update, 0, 1);
    };
}
