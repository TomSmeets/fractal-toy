// Sorry, but these warnings are very annoying
#![allow(dead_code)]
#![allow(unused_variables)]

mod gpu;
mod util;
mod tilemap;

use cgmath::Vector2;
use gpu::{Gpu, GpuInput};
use tilemap::TilePos;
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

pub struct Input {
    resolution: Vector2<u32>,
    mouse: Vector2<i32>,
}

pub struct State {
    gpu: Gpu,
}

pub struct Image {
    size: Vector2<u32>,
    data: Vec<u8>,
}

impl State {
    pub fn init() -> Self {
        State {
            gpu: Gpu::init(),
        }
    }

    pub fn gen_tile(&mut self, p: TilePos) -> Image {
        let size = 7;
        let mut data = Vec::with_capacity(size as usize * size as usize * 4);
        for y in 0..size {
            for x in 0..size {
                let border = (y == 0 || y == size  - 1) || (x == 0 || x == size-1);
                data.push(if border { 255 } else { 0 });
                data.push(if border { 255 } else { 0 });
                data.push(if border { 255 } else { 0 });
                data.push(255);
            }
        }

        Image { size: Vector2::new(size, size), data }
    }

    /// always called at regular intervals
    pub fn update(&mut self, window: &Window, input: &Input, dt: f32) {
        let mut tiles_todo = Vec::new();
        let mut low = Vector2::new(input.mouse.x as f64 / input.resolution.x as f64, input.mouse.y as f64 / input.resolution.y as f64);
        low.y = 1.0 - low.y;
        for i in 0..4 {
            tiles_todo.extend(TilePos::between(low, Vector2::new(0.9, 0.9), i, 1));
        }

        let tiles = tiles_todo.into_iter().map(|x| (x, self.gen_tile(x))).collect::<Vec<_>>();

        // tiles.extend(TilePos::between(Vector2::new(0.3, 0.3), Vector2::new(0.7, 0.7), 3, 0));
        self.gpu.render(window, &GpuInput {
            resolution: input.resolution,
            tiles: &tiles,
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

    let resolution = window.inner_size();
    let mut input = Input {
        resolution: Vector2::new(resolution.width, resolution.height),
        mouse: Vector2::new(0, 0),
    };

    let mut state = State::init();

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
                input.resolution.x = size.width;
                input.resolution.y = size.height;
            },

            Event::WindowEvent {
                window_id: _,
                event: WindowEvent::CursorMoved { position: pos, ..},
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
                    state.update(&window, &input, target_dt);

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
