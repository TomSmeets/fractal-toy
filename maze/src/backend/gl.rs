use super::Config;
use crate::core::Maze;
use crate::core::MazeBuilder;
use crate::core::Tile;
use gilrs::Gilrs;
use glutin::event::Event;
use glutin::event::VirtualKeyCode;
use glutin::event::WindowEvent;
use glutin::event_loop::ControlFlow;
use glutin::event_loop::EventLoop;
use glutin::window::Window;
use glutin::window::WindowBuilder;
use glutin::{ContextBuilder, ContextWrapper, PossiblyCurrent};

mod raw;
use raw as gl;
use raw::Gl;

pub fn run(cfg: Config) {
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

    let mut size = [800, 600];

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        if let Event::WindowEvent { event, .. } = &event {
            match event {
                WindowEvent::Resized(s) => size = [s.width as i32, s.height as i32],
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                    Some(VirtualKeyCode::Q) => *control_flow = ControlFlow::Exit,
                    Some(VirtualKeyCode::I) => {}
                    _ => (),
                },
                _ => (),
            }
        }
        if let Event::MainEventsCleared { .. } = &event {
            unsafe {
                gl.ClearColor(1.0, 0.0, 1.0, 1.0);
                gl.Clear(gl::COLOR_BUFFER_BIT);
                gl.Viewport(0, 0, size[0], size[1]);
            }

            ctx.swap_buffers().unwrap();
        }
    });
}
