use gilrs::Gilrs;
use glutin::event_loop::EventLoop;
use glutin::window::Window;
use glutin::window::WindowBuilder;
use glutin::{ContextBuilder, ContextWrapper, PossiblyCurrent};

use crate::gl::Gl;

use std::sync::Arc;
use std::sync::Mutex;

pub struct Platform {
    pub gilrs: Gilrs,
    pub gl: Gl,
    pub ctx: ContextWrapper<PossiblyCurrent, Window>,
}

impl Platform {
    pub fn new() -> (Platform, EventLoop<()>) {
        let event_loop = EventLoop::new();
        let window_builder = WindowBuilder::new().with_title("dev");

        let ctx = ContextBuilder::new()
            .with_vsync(true)
            .build_windowed(window_builder, &event_loop)
            .unwrap();
        let ctx: ContextWrapper<PossiblyCurrent, _> = unsafe { ctx.make_current().unwrap() };

        // glutin is mostly the same api as winit
        let gl = Gl::load_with(|s| ctx.get_proc_address(s));
        let gilrs = Gilrs::new().unwrap();
        (Platform { ctx, gl, gilrs }, event_loop)
    }
}
