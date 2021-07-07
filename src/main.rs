// Sorry, but these warnings are very annoying
#![allow(dead_code)]
#![allow(unused_variables)]

mod asset_loader;
mod builder;
mod gpu;
mod image;
mod pack;
mod tilemap;
mod util;
mod viewport;
mod debug;

use self::asset_loader::AssetLoader;
use self::builder::TileBuilder;
use self::gpu::Gpu;
use self::image::Image;
use self::tilemap::TilePos;
use self::util::*;
use self::viewport::Viewport;

use std::collections::BTreeMap;
use std::process::Command;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
use debug::Debug;
use structopt::StructOpt;
use winit::event::WindowEvent;
use winit::event::{ElementState, Event, MouseButton, MouseScrollDelta};
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

#[derive(Debug)]
pub struct Input {
    resolution: V2<u32>,
    mouse: V2<i32>,
    mouse_down: bool,
    mouse_scroll: f32,
}

pub struct State {
    gpu: Gpu,
    builder: TileBuilder,
    asset: AssetLoader,
    debug: Debug,

    // actual state that is relevant
    viewport: Viewport,
}

impl State {
    pub fn init(window: &Window) -> Self {
        let mut asset = AssetLoader::new();
        let gpu = Gpu::init(window, &mut asset);
        let builder = TileBuilder::new(gpu.device(), &mut asset);
        State {
            debug: Debug::new(),
            gpu,
            builder,
            viewport: Viewport::new(),
            asset,
        }
    }

    pub fn distance(scale: f64) -> String {
        let mut result = String::new();
        let scales = [
            ("*10^6 km", 1e9),
            ("*10^3 km", 1e6),
            ("km", 1e3),
            (" m", 1e1),
            ("mm", 1e-3),
            ("um", 1e-6),
            ("nm", 1e-9),
            ("pm", 1e-12),
        ];

        // TODO: visual scale indicator,
        // Small solarsystem -> eart -> tree -> etc
        let objects = [
            ("solar system", 8.99683742e12),
            ("the sun", 1.391e9),
            ("earth", 1.2742018e7),
            ("europe", 13791e3),
            ("The Netherlands", 115e3),
            ("City", 6.3e3),
            ("Street", 146.0),
            ("House", 16.0),
        ];

        let size_meters = scale * 9e12;

        for (n, s) in scales.iter() {
            if size_meters > *s {
                result += &format!("{:6.2} {}\n", size_meters / s, n);
                break;
            }
        }

        for (n, s) in objects.iter().rev() {
            if size_meters <= *s * 2.0 {
                result += &format!(" {:6.1} x {}", size_meters / s, n);
                break;
            }
        }

        result
    }

    /// always called at regular intervals
    pub fn update(&mut self, window: &Window, input: &Input, dt: f32) {
        self.debug.time("viewport");
        // viewport stuff
        self.viewport.size(input.resolution);
        self.viewport
            .zoom_at(input.mouse_scroll as f64, input.mouse);

        if input.mouse_down {
            self.viewport.drag(input.mouse);
        }

        self.viewport.update(dt as f64);

        self.debug.time("build tiles");
        // which tiles to build
        for p in self.viewport.get_pos_all(1) {
            self.builder.tile(&p);
        }

        self.debug.time("upload gpu tiles");
        // which tiles to draw
        for p in self.viewport.get_pos_all(0) {
            if let Some(img) = self.builder.tile(&p) {
                self.gpu.tile(&self.viewport, &p, img);
            }
        }

        // submit
        self.debug.time("debug text");
        self.debug.print(&Self::distance(self.viewport.scale));
        self.asset.text(&mut self.gpu, &self.debug.draw());

        self.debug.time("gpu render");
        self.gpu.render(window, &self.viewport, &mut self.debug);

        self.debug.time("builder update");
        self.builder.update();
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
                .arg("-r")
                .arg(id.to_string())
                .arg("-t")
                .arg(ws.to_string())
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

    let mut state = State::init(&window);

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
                    state.debug.begin();
                    state.debug.time("state.update (start)");
                    state.update(&window, &input, target_dt);
                    state.debug.time("state.update (end)");
                    input.mouse_scroll = 0.0;

                    // check how accurate we actually are
                    // TODO: extract to timing struct
                    // if config.debug {
                        let dt_frame = current_time - last_frame_time;
                        let dt_behind = current_time - next_frame_time;
                        let dt_update = Instant::now() - current_time;
                        let rate = format!(
                            "{:.1} Hz\nframe {:6?} µs, update {:6} µs, behind {:2?} µs",
                            1.0 / dt_frame.as_secs_f32(),
                            dt_frame.as_micros(),
                            dt_update.as_micros(),
                            dt_behind.as_micros()
                        );
                        state.debug.print(&rate);
                        last_frame_time = current_time;
                    // }

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
