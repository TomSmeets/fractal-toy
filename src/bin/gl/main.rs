use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use serial::atlas::*;
use serial::math::*;
use serial::time::DeltaTime;
use serial::Fractal;
use serial::Input;

mod ctx;
mod gl;
mod imm;
mod program;
mod shader;

use self::ctx::GLCtx;
use self::gl::Gl;

static mut GL: Option<Gl> = None;

unsafe fn static_gl() -> &'static mut Gl {
    match &mut GL {
        Some(x) => x,
        None => panic!(),
    }
}

fn handle_input(input: &mut Input, event: &WindowEvent) {
    use glutin::event;
    use glutin::event::ElementState::*;
    use glutin::event::MouseButton::*;
    use glutin::event::MouseScrollDelta::*;
    use glutin::event::VirtualKeyCode as VK;
    use glutin::event::WindowEvent::*;

    match event {
        CursorMoved { position, .. } => {
            input.mouse = Vector2::new(position.x as _, position.y as _)
        },
        MouseWheel { delta, .. } => match delta {
            LineDelta(_, y) => input.scroll = *y as i32,
            PixelDelta(_) => (),
        },
        MouseInput { button, state, .. } => match button {
            Left => input.mouse_down = *state == Pressed,
            _ => (),
        },
        KeyboardInput {
            input:
                event::KeyboardInput {
                    virtual_keycode: Some(key),
                    state,
                    ..
                },
            ..
        } => {
            let f = match *state {
                Pressed => 1.0,
                Released => 0.0,
            };

            match key {
                VK::W => input.dir_move.y = f,
                VK::S => input.dir_move.y = -f,
                VK::A => input.dir_move.x = -f,
                VK::D => input.dir_move.x = f,
                _ => {
                    dbg!(key);
                },
            }
        },
        _ => (),
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new();

    let mut ctx = GLCtx::new(window, &event_loop);
    let mut fractal: Fractal<AtlasRegion> = Fractal::new(Vector2::new(800, 600));
    let mut input = Input::new();
    let dt = DeltaTime(1.0 / 60.0);
    let mut atlas = Atlas::new();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(sz) => {
                    fractal.pos.resize(Vector2::new(sz.width, sz.height));
                    ctx.resize(sz.width, sz.height);
                },
                e => handle_input(&mut input, &e),
            },
            Event::MainEventsCleared => {
                fractal.do_input(&input, dt);
                input.begin();
                {
                    let mut p = imm::Provider { gl: ctx.gl() };
                    let mut p = AtlasTextureCreator {
                        atlas: &mut atlas,
                        sdl: &mut p,
                    };
                    fractal.update_tiles(&mut p);
                }
                ctx.draw(&atlas, &fractal);
            },
            Event::RedrawRequested(_) => {},
            _ => (),
        }
    });
}
