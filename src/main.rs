// Sorry, but these warnings are very annoying
#![allow(dead_code)]
#![allow(unused_variables)]

mod gpu;
mod util;
mod tilemap;
mod viewport;
mod builder;

use self::builder::TileBuilder;
use self::gpu::{Gpu, GpuInput};
use self::tilemap::TilePos;
use self::viewport::{Viewport, ViewportInput};
use self::util::*;

use std::process::Command;
use std::time::Duration;
use std::time::Instant;
use structopt::StructOpt;
use winit::event::{ElementState, Event, MouseButton, MouseScrollDelta};
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
    resolution: V2<u32>,
    mouse: V2<i32>,
    mouse_down: bool,
    mouse_scroll: f32,
}

pub struct State {
    gpu: Gpu,
    builder: TileBuilder,

    // actual state that is relevant
    viewport: Viewport,
    drag: Option<V2<f64>>,
}

pub struct Image {
    size: V2<u32>,
    data: Vec<u8>,
}

impl State {
    pub fn init() -> Self {
        State {
            gpu: Gpu::init(),
            builder: TileBuilder::new(),
            viewport: Viewport::new(),
            drag: None,
        }
    }

    /// always called at regular intervals
    pub fn update(&mut self, window: &Window, input: &Input, dt: f32) {

        let vp = self.viewport.update(dt as f64, &ViewportInput {
            resolution: input.resolution,
            zoom: (input.mouse_scroll as f64, input.mouse),
            world2screen: self.drag.map(|d| (d, input.mouse)),
        });

        if input.mouse_down && self.drag.is_none() {
            self.drag = Some(vp.screen_to_world(input.mouse));
        }

        if !input.mouse_down && self.drag.is_some() {
            self.drag = None;
        }

        let mut todo = Vec::new();

        // build padded
        vp.get_pos_all(&mut todo, 1);
        let cache = self.builder.build(&todo);

        // display unpadded
        todo.clear();
        vp.get_pos_all(&mut todo, 0);
        let tiles = todo.iter().flat_map(|k| cache.get_key_value(k)).collect::<Vec<_>>();

        dbg!(cache.len());
        dbg!(tiles.len());

        self.gpu.render(window, &GpuInput {
            resolution: input.resolution,
            viewport: &vp,
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
        resolution: V2::new(resolution.width, resolution.height),
        mouse: V2::new(0, 0),
        mouse_down: false,
        mouse_scroll: 0.0,
    };

    let mut state = State::init();

    // Decide what framerate we want to run
    let target_dt = 1.0 / 180.0;

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
                event: WindowEvent::MouseInput { button, state, .. },
            } => {
                if button == MouseButton::Left {
                    input.mouse_down = state == ElementState::Pressed;
                }
            },

            Event::WindowEvent {
                window_id: _,
                event: WindowEvent::MouseWheel { delta: MouseScrollDelta::LineDelta(dx, dy), ..},
            } => {
                input.mouse_scroll += dy;
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
                    input.mouse_scroll = 0.0;

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

                    while next_frame_time < current_time {
                        next_frame_time += Duration::from_secs_f32(target_dt);
                    }
                }

                *control_flow = ControlFlow::WaitUntil(next_frame_time);
            },
            _ => (),
        }
    });
}
