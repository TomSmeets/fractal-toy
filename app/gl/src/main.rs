mod atlas;
mod ctx;
mod gl;
mod imm;
mod program;
mod shader;
mod texture;

use self::atlas::Atlas;
use self::ctx::GLCtx;
use self::gl::Gl;
use fractal_toy::math::*;
use fractal_toy::AtlasRegion;
use fractal_toy::DeltaTime;
use fractal_toy::Fractal;
use fractal_toy::Input;
use fractal_toy::UIInput;
use fractal_toy::UI;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use imgui::*;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::time::Instant;

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

    if let Ok(b) = builder_ocl::OCLWorker::new() {
        fractal.add_builder(|| b);
    }

    let mut input = Input::new();
    let dt = DeltaTime(1.0 / 60.0);
    let mut atlas = Atlas::new();

    let mut imgui = Context::create();
    let mut gui = UI::new();
    let mut platform = WinitPlatform::init(&mut imgui); // step 1
    platform.attach_window(imgui.io_mut(), ctx.ctx.window(), HiDpiMode::Default); // step 2
    let renderer =
        imgui_opengl_renderer::Renderer::new(&mut imgui, |s| ctx.ctx.get_proc_address(s) as _);

    let mut last_frame = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        platform.handle_event(imgui.io_mut(), ctx.ctx.window(), &event);
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(sz) => {
                    fractal.pos.resize(Vector2::new(sz.width, sz.height));
                    ctx.resize(sz.width, sz.height);
                },
                e => {
                    if !imgui.io().want_capture_mouse && !imgui.io().want_capture_keyboard {
                        handle_input(&mut input, &e);
                    }
                },
            },
            Event::MainEventsCleared => {
                last_frame = imgui.io_mut().update_delta_time(last_frame);

                platform
                    .prepare_frame(imgui.io_mut(), ctx.ctx.window())
                    .unwrap();

                input.execute(&mut fractal, dt);
                input.begin();
                fractal.update_tiles(&mut atlas.provider(ctx.gl()));

                let ui_input = UIInput {
                    viewport: V2i::new(ctx.size.x as i32, ctx.size.y as i32),
                    mouse: input.mouse,
                    left: input.mouse_down,
                    right: false,
                };

                gui.input(ui_input);
                gui.update(&mut fractal);

                let ui = imgui.frame();

                Window::new(im_str!("Hello world"))
                    .size([300.0, 100.0], Condition::FirstUseEver)
                    .build(&ui, || {
                        ui.checkbox(im_str!("debug"), &mut input.debug);
                        ui.checkbox(im_str!("pause"), &mut input.pause);

                        let mut offset = [fractal.pos.offset.x as f32, fractal.pos.offset.y as f32];
                        ui.drag_float2(im_str!("offset"), &mut offset).build();
                        // fractal.pos.offset.x = offset[0] as f64;
                        // fractal.pos.offset.y = offset[1] as f64;

                        Slider::new(im_str!("zoom"), 0.0..=48.5).build(&ui, &mut fractal.pos.zoom);
                        use fractal_toy::InputAction;
                        use fractal_toy::InputEvent;
                        if ui.button(im_str!("Iter+"), [60.0, 30.0]) {
                            input
                                .events
                                .push(InputEvent::Action(InputAction::IterInc, true));
                            input
                                .events
                                .push(InputEvent::Action(InputAction::IterInc, false));
                        }
                        if ui.button(im_str!("Iter-"), [60.0, 30.0]) {
                            input
                                .events
                                .push(InputEvent::Action(InputAction::IterDec, true));
                            input
                                .events
                                .push(InputEvent::Action(InputAction::IterDec, false));
                        }

                        if ui.button(im_str!("next"), [60.0, 30.0]) {
                            input
                                .events
                                .push(InputEvent::Action(InputAction::NextFractal, true));
                            input
                                .events
                                .push(InputEvent::Action(InputAction::NextFractal, false));
                        }
                    });

                platform.prepare_render(&ui, ctx.ctx.window());

                let gl = ctx.gl();
                unsafe {
                    gl.ClearColor(1.0, 1.0, 1.0, 0.0);
                    gl.Clear(gl::COLOR_BUFFER_BIT);
                }

                ctx.draw(&gui, &atlas, &fractal);
                renderer.render(ui);

                ctx.ctx.swap_buffers().unwrap();
            },
            Event::RedrawRequested(_) => {},
            _ => (),
        }
    });
}
