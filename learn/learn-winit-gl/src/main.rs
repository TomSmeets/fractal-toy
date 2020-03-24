mod gl;

use glutin::event::Event;
use glutin::event::WindowEvent;
use glutin::event_loop::ControlFlow;
use glutin::event_loop::EventLoop;
use glutin::window::Window;
use glutin::window::WindowBuilder;
use glutin::{ContextWrapper, PossiblyCurrent};

use std::time::Instant;

struct Platform {
    gl: gl::Gl,
    ctx: ContextWrapper<PossiblyCurrent, Window>,
}

struct State {
    quit: bool,
    time: Instant,
}

impl State {
    fn new() -> State {
        State {
            quit: false,
            time: Instant::now(),
        }
    }

    fn event(&mut self, event: &Event<()>) {
        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            self.quit = true;
        }
    }

    fn update(&mut self, platform: &mut Platform) {
        let dt = {
            let time_now = Instant::now();
            let dt = (time_now - self.time).as_secs_f32();
            let fps = 1.0 / dt;
            dbg!(dt, fps);
            self.time = time_now;
            dt
        };

        unsafe {
            let gl = &mut platform.gl;
            gl.ClearColor(1.0, 0.0, 1.0, 1.0);
            gl.Clear(gl::COLOR_BUFFER_BIT);
        }
        platform.ctx.swap_buffers().unwrap();
    }
}

impl Platform {
    pub fn new() -> (Platform, EventLoop<()>) {
        let event_loop = EventLoop::new();
        let window_builder = WindowBuilder::new();

        let ctx = glutin::ContextBuilder::new()
            .with_vsync(true)
            .build_windowed(window_builder, &event_loop)
            .unwrap();
        let ctx: ContextWrapper<PossiblyCurrent, _> = unsafe { ctx.make_current().unwrap() };

        // glutin is mostly the same api as winit
        let gl = gl::Gl::load_with(|s| ctx.get_proc_address(s) as *const _);
        (Platform { ctx, gl }, event_loop)
    }
}

fn main() {
    println!("Hello, world!");
    let (mut platform, event_loop) = Platform::new();
    let mut state = State::new();
    event_loop.run(move |event, _, control_flow| {
        state.event(&event);
        if state.quit {
            *control_flow = ControlFlow::Exit;
        } else {
            *control_flow = ControlFlow::Poll;
        }
        match event {
            Event::WindowEvent { event, .. } => {
                dbg!(event);
            }
            Event::MainEventsCleared { .. } => {
                state.update(&mut platform);
            }
            _ => (),
        }
    });
}
