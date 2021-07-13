use crate::util::*;
use ::instant::Duration;
use ::instant::Instant;
use winit::event::WindowEvent;
use winit::event::{ElementState, Event, MouseButton, MouseScrollDelta};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;
use winit::window::WindowBuilder;

#[derive(Debug)]
pub struct Input {
    pub dt: f32,
    pub real_dt_full: Duration,
    pub real_dt_update: Duration,
    pub resolution: V2<u32>,

    pub mouse: V2<i32>,
    pub mouse_down: bool,
    pub mouse_click: bool,

    pub mouse_scroll: f32,
}

pub struct Loop {
    pub event_loop: EventLoop<()>,
    pub window: Window,
}

impl Loop {
    pub fn new(title: &'static str) -> Self {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(title)
            .build(&event_loop)
            .unwrap();

        Loop { event_loop, window }
    }

    pub fn run<F: FnMut(&Window, &Input) + 'static>(self, mut update: F) -> ! {
        // Decide what framerate we want to run
        let target_dt = 1.0 / 180.0;

        let resolution = self.window.inner_size();
        let mut input = Input {
            dt: target_dt,
            real_dt_full: Duration::ZERO,
            real_dt_update: Duration::ZERO,
            resolution: V2::new(resolution.width, resolution.height),
            mouse: V2::new(0, 0),
            mouse_down: false,
            mouse_click: false,
            mouse_scroll: 0.0,
        };

        // At what time do we want a new update
        let mut next_frame_time = Instant::now();
        let mut last_frame_time = Instant::now();

        // NOTE: we are ignoring redraw requests for now,
        // and are both updating and rendering in MainEventsCleared.
        // This might result into issues in the web platform,
        // but lets keep it as simple as possible.
        let event_loop = self.event_loop;
        let window = self.window;
        event_loop.run(move |event, _, control_flow| {
            // TODO: Poll vs Wait, what is usefull?
            // NOTE: using just Wait, will not work. why?
            *control_flow = ControlFlow::WaitUntil(next_frame_time);

            match event {
                // Respect window close button
                Event::WindowEvent {
                    window_id: _,
                    event: WindowEvent::CloseRequested,
                } => *control_flow = ControlFlow::Exit,

                Event::WindowEvent {
                    window_id: _,
                    event: WindowEvent::Resized(size),
                } => {
                    input.resolution.x = size.width;
                    input.resolution.y = size.height;
                },

                Event::WindowEvent {
                    window_id: _,
                    event: WindowEvent::MouseInput { button, state, .. },
                } => {
                    if button == MouseButton::Left {
                        let was_down = input.mouse_down;
                        let is_down = state == ElementState::Pressed;
                        input.mouse_down = is_down;
                        input.mouse_click = is_down;
                    }
                },

                Event::WindowEvent {
                    window_id: _,
                    event:
                        WindowEvent::MouseWheel {
                            delta: MouseScrollDelta::LineDelta(dx, dy),
                            ..
                        },
                } => {
                    input.mouse_scroll += dy;
                },

                Event::WindowEvent {
                    window_id: _,
                    event: WindowEvent::CursorMoved { position: pos, .. },
                } => {
                    input.mouse.x = pos.x as _;
                    input.mouse.y = pos.y as _;
                },

                // After all events are handled, time to update.
                // or not, if this is called on any event that might have happened
                Event::MainEventsCleared => {
                    // check if we shoud update
                    let current_time = Instant::now();

                    // NOTE: if it was a while loop it would loop forever if we couldent keep up
                    // now it still requests an instaint update, but gives the os some cpu time
                    if next_frame_time <= current_time {
                        update(&window, &input);
                        input.real_dt_full = current_time - last_frame_time;
                        input.real_dt_update = Instant::now() - current_time;
                        last_frame_time = current_time;
                        input.mouse_scroll = 0.0;
                        input.mouse_click = false;

                        while next_frame_time < current_time {
                            next_frame_time += Duration::from_secs_f32(target_dt);
                        }
                    }

                    *control_flow = ControlFlow::WaitUntil(next_frame_time);
                },
                _ => (),
            }
        })
    }
}
