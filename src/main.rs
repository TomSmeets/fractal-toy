mod gpu;

use std::time::Instant;
use std::time::Duration;

use gpu::Gpu;
use winit::window::Window;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::event::Event;
use winit::event::WindowEvent;

pub struct State {
    // Handle to the gpu
    gpu: Gpu,
}

impl State {
    pub fn init(window: &Window) -> Self {
        State {
            gpu: Gpu::init(window),
        }
    }

    /// always called at regular intervals
    pub fn update(&mut self, dt: f32) {}

    /// Can sometimes be called more often than update
    pub fn render(&mut self) {}
}

pub fn main() {
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();
    let mut state = State::init(&window);

    // Decide what framerate we want to run
    let target_dt = 1.0 / 60.0;

    
    // At what time do we want a new update
    let mut next_frame_time = Instant::now();
    let mut last_frame_time = Instant::now();
    
    // NOTE: we are ignoring redraw requests for now,
    // and are both updating and rendering in MainEventsCleared.
    // This might result into issues in the web platform,
    // but lets keep it as simple as possible.
    event_loop.run(move |event, _, control_flow| {
        // TODO: Poll vs Wait, what is usefull?
        // whaterver happens
        *control_flow = ControlFlow::WaitUntil(next_frame_time);

        match event {
            // Respect window close button
            Event::WindowEvent {
                window_id: _,
                event: WindowEvent::CloseRequested
            } => *control_flow = ControlFlow::Exit,

            // After all events are handled, time to update.
            // or not, if this is called on any event that might have happened
            Event::MainEventsCleared => {
                // check if we shoud update
                let current_time    = Instant::now();

                // NOTE: if it was a while loop it would loop forever if we couldent keep up
                // now it still requests an instaint update, but gives the os some cpu time
                if next_frame_time <= current_time {
                    // check how accurate we actually are
                    if false {
                        let dt = current_time - last_frame_time;
                        println!("update: {:.2}, {:.2?}", 1.0 / dt.as_secs_f32(), dt);
                        last_frame_time = current_time;
                    }

                    state.update(target_dt);
                    state.render();
                    next_frame_time += Duration::from_secs_f32(target_dt);
                }

                *control_flow = ControlFlow::WaitUntil(next_frame_time);
            },
            _ => (),
        }
    });
}
