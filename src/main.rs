// Sorry, but these warnings are very annoying
#![allow(dead_code)]
#![allow(unused_variables)]

mod gpu;
mod util;

use cgmath::Vector2;
use gpu::{Gpu, GpuInput};
use std::process::Command;
use std::time::Duration;
use std::time::Instant;
use structopt::StructOpt;
use winit::event::Event;
use winit::event::WindowEvent;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;
use winit::window::WindowBuilder;

#[derive(Debug, StructOpt)]
struct Config {
    #[structopt(short, long)]
    move_window: Option<Option<u32>>,

    #[structopt(short, long)]
    debug: bool,
}

pub struct State {
    resolution: Vector2<u32>,
    gpu: Gpu,
}

impl State {
    pub fn init(window: &Window) -> Self {
        let resolution = window.inner_size();
        State {
            resolution: Vector2::new(resolution.width, resolution.height),
            gpu: Gpu::init(),
        }
    }

    /// always called at regular intervals
    pub fn update(&mut self, window: &Window, dt: f32) {
        self.gpu.render(window, &GpuInput {
            resolution: self.resolution,
        });
    }
}

pub fn main() {
    let config = Config::from_args();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Fractal Toy!")
        .build(&event_loop)
        .unwrap();

    dbg!(&config);
    if let Some(ws) = config.move_window {
        let ws = ws.unwrap_or(9);

        use winit::platform::unix::WindowExtUnix;
        // very hacky way to move the window out of my way
        // when using 'cargo watch -x run'
        // was to lazy to modify my wm or so.
        // This actually works very well :)
        if let Some(id) = window.xlib_window() {
            let _ = Command::new("wmctrl")
                .arg("-i")
                .arg("-r").arg(id.to_string())
                .arg("-t").arg(ws.to_string())
                .status();
        }
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
                event: WindowEvent::CloseRequested,
            } => *control_flow = ControlFlow::Exit,

            Event::WindowEvent {
                window_id: _,
                event: WindowEvent::Resized(size),
            } => {
                state.resolution.x = size.width;
                state.resolution.y = size.height;
            },

            // After all events are handled, time to update.
            // or not, if this is called on any event that might have happened
            Event::MainEventsCleared => {
                // check if we shoud update
                let current_time = Instant::now();

                // NOTE: if it was a while loop it would loop forever if we couldent keep up
                // now it still requests an instaint update, but gives the os some cpu time
                if next_frame_time <= current_time {
                    state.update(&window, target_dt);

                    // check how accurate we actually are
                    // TODO: extract to timing struct
                    if config.debug {
                        let dt_frame  = current_time - last_frame_time;
                        let dt_behind = current_time - next_frame_time;
                        let dt_update = Instant::now() - current_time;
                        println!(
                            "{:.1} Hz, frame {:6?} µs, update {:6} µs, behind {:2?} µs",
                            1.0 / dt_frame.as_secs_f32(),
                            dt_frame.as_micros(),
                            dt_update.as_micros(),
                            dt_behind.as_micros()
                        );
                        last_frame_time = current_time;
                    }

                    next_frame_time += Duration::from_secs_f32(target_dt);
                }

                *control_flow = ControlFlow::WaitUntil(next_frame_time);
            },
            _ => (),
        }
    });
}
