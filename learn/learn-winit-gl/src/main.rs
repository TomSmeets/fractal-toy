mod gl;

use glutin::event::Event;
use glutin::event_loop::ControlFlow;
use glutin::event_loop::EventLoop;
use glutin::window::WindowBuilder;
use glutin::{ContextWrapper, PossiblyCurrent};

use std::time::Instant;

fn main() {
    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new();

    let ctx = glutin::ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(window_builder, &event_loop)
        .unwrap();
    let ctx: ContextWrapper<PossiblyCurrent, _> = unsafe { ctx.make_current().unwrap() };

    // glutin is mostly the same api as winit
    let gl = gl::Gl::load_with(|s| ctx.get_proc_address(s) as *const _);
    println!("Hello, world!");
    let mut time_old = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        // println!("event: {:?}", event);

        match event {
            Event::WindowEvent { event, .. } => {
                dbg!(event);
            }
            Event::MainEventsCleared { .. } => {
                let time_now = Instant::now();
                let dt = (time_now - time_old).as_secs_f32();
                {
                    let fps = 1.0 / dt;
                    dbg!(dt, fps);
                }
                time_old = time_now;
                unsafe {
                    gl.ClearColor(1.0, 0.0, 1.0, 1.0);
                    gl.Clear(gl::COLOR_BUFFER_BIT);
                }
                ctx.swap_buffers().unwrap();
            }
            _ => (),
        }
    });
}
