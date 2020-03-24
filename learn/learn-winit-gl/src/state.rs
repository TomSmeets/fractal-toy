use glutin::event::Event;
use glutin::event::WindowEvent;
use std::time::Instant;

use crate::gl;
use crate::Platform;

pub struct State {
    pub quit: bool,

    time: Instant,
    dt: f32,
}

impl State {
    pub fn new() -> State {
        State {
            dt: 0.0,
            quit: false,
            time: Instant::now(),
        }
    }

    pub fn event(&mut self, event: &Event<()>) {
        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            self.quit = true;
        }
    }

    pub fn update(&mut self, platform: &mut Platform) {
        {
            let time_now = Instant::now();
            let dt = (time_now - self.time).as_secs_f32();
            self.time = time_now;
            self.dt = dt;
        }

        unsafe {
            let gl = &mut platform.gl;
            gl.ClearColor(1.0, 0.0, 1.0, 1.0);
            gl.Clear(gl::COLOR_BUFFER_BIT);
        }
        platform.ctx.swap_buffers().unwrap();
    }
}
