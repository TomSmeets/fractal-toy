use glutin::event::Event;
use glutin::event::VirtualKeyCode;
use glutin::event::WindowEvent;
use std::time::Instant;

use crate::gfx::imm::GfxImmState;
use crate::gl;
use crate::gl::types::*;
use crate::gl::Gl;
use crate::Platform;

#[derive(Debug)]
pub struct State {
    pub quit: bool,
    pub time: Instant,
    pub dt: f32,
    pub imm: GfxImmState,

    pub size: [i32; 2],
}

impl State {
    pub fn new(platform: &mut Platform) -> State {
        let imm = GfxImmState::new(&platform.gl);
        dbg!(&imm);
        State {
            dt: 0.0,
            quit: false,
            time: Instant::now(),
            imm,
            size: [800, 600],
        }
    }

    pub fn event(&mut self, event: &Event<()>) {
        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::Resized(s) => self.size = dbg!([s.width as _, s.height as _]),
                WindowEvent::CloseRequested => self.quit = true,
                WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                    Some(VirtualKeyCode::Q) => self.quit = true,
                    Some(VirtualKeyCode::I) => {
                        println!("{:#?}", self);
                    }
                    _ => (),
                },
                _ => (),
            }
        }
    }

    pub fn update(&mut self, platform: &mut Platform) {
        {
            let time_now = Instant::now();
            let dt = (time_now - self.time).as_secs_f32();
            self.time = time_now;
            self.dt = dt;
        }

        while let Some(gilrs::Event { id, event, time }) = platform.gilrs.next_event() {
            println!("{:?} New event from {}: {:?}", time, id, event);
        }

        let gl = &mut platform.gl;
        unsafe {
            gl.ClearColor(1.0, 0.0, 1.0, 1.0);
            gl.Clear(gl::COLOR_BUFFER_BIT);
            gl.Viewport(0, 0, self.size[0], self.size[1]);
        }

        self.imm.draw();

        platform.ctx.swap_buffers().unwrap();
    }
}
