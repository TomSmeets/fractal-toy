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

pub mod raw;

pub mod imm;
pub mod program;
pub mod shader;

use imm::GfxImmState;
use raw as gl;
use raw::Gl;

pub static mut GL: Option<Gl> = None;

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

    let mut gen = MazeBuilder::new(cfg.width, cfg.height);

    let mut imm = GfxImmState::new(&gl);

    unsafe {
        GL = Some(gl);
    }

    event_loop.run(move |event, _, control_flow| {
        let gl = unsafe { GL.as_ref().unwrap() };
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

            let s = gen.maze.size();
            let w = 1.0 / s[0] as f32;
            let h = 1.0 / s[1] as f32;
            for j in 0..s[1] {
                for i in 0..s[0] {
                    let x = i as f32 * w;
                    let y = j as f32 * h;
                    let tile = gen.maze.at((i, j));
                    let c = match tile {
                        Tile::Empty => 0.8,
                        Tile::Wall => 0.2,
                        Tile::Undefined => 0.0,
                    };
                    imm.rect([x, y, w, h], [c, c, c]);
                }
            }

            for &(i, j) in gen.queue.iter() {
                let x = i as f32 * w;
                let y = j as f32 * h;
                imm.rect([x, y, w, h], [0.0, 0.0, 1.0]);
            }

            imm.draw(&gl);

            for _ in 0..2 {
                gen.next();
            }

            ctx.swap_buffers().unwrap();
        }
    });
}
