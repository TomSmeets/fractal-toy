use gilrs;
use gilrs::Gilrs;
use glutin::event::Event;
use glutin::event::WindowEvent;
use std::time::Instant;

use crate::gl;
use crate::Platform;

pub struct State {
    pub quit: bool,
    pub time: Instant,
    pub dt: f32,
    pub gilrs: Gilrs,
}

impl State {
    pub fn new() -> State {
        let gilrs = Gilrs::new().unwrap();
        State {
            dt: 0.0,
            quit: false,
            time: Instant::now(),
            gilrs,
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

        while let Some(gilrs::Event { id, event, time }) = self.gilrs.next_event() {
            println!("{:?} New event from {}: {:?}", time, id, event);
        }

        unsafe {
            let gl = &mut platform.gl;
            gl.ClearColor(1.0, 0.0, 1.0, 1.0);
            gl.Clear(gl::COLOR_BUFFER_BIT);
        }
        platform.ctx.swap_buffers().unwrap();
    }
}
