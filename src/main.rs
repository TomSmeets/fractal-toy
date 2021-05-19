// Sorry, but these warnings are very annoying
#![allow(dead_code)]
#![allow(unused_variables)]

mod gpu;

use std::process::Command;
use std::time::Instant;
use std::time::Duration;

use gpu::Gpu;
use winit::window::Window;
use winit::window::WindowBuilder;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::event::Event;
use winit::event::WindowEvent;

pub struct State {
    gpu: Gpu,
}

impl State {
    pub fn init(window: &Window) -> Self {
        State {
            gpu: Gpu::init(window),
        }
    }

    /// always called at regular intervals
    pub fn update(&mut self, dt: f32) {
    }
}

pub fn main() {
    let title = "Fractaly Toy!";
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_title(title).build(&event_loop).unwrap();

    {
        // very hacky way to move the window out of my way
        // when using 'cargo watch -x run'
        // was to lazy to modify my wm or so.
        // This actually works very well :)
        //
        // NOTE: Sadly we had to use the window title, and hope that it is uniqe.
        // I would like to use the x11 window id, but winit does not expose it to me.
        Command::new("wmctrl").arg("-r").arg(title).arg("-t").arg("9").status().unwrap();
    }

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
        // NOTE: using just Wait, will not work. why?
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
                let current_time = Instant::now();

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
                    next_frame_time += Duration::from_secs_f32(target_dt);
                }

                *control_flow = ControlFlow::WaitUntil(next_frame_time);
            },
            _ => (),
        }
    });
}
