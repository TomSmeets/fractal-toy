// Sorry, but these warnings are very annoying
#![allow(dead_code)]
#![allow(unused_variables)]

mod gpu;
mod util;
mod tilemap;
mod viewport;

use cgmath::Vector2;
use self::gpu::{Gpu, GpuInput};
use self::tilemap::TilePos;
use self::viewport::{Viewport, ViewportInput};
use std::{collections::BTreeMap, process::Command};
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
    resolution: Vector2<u32>,
    mouse: Vector2<i32>,
    mouse_down: bool,
    mouse_scroll: f32,
}

pub struct State {
    viewport: Viewport,
    gpu: Gpu,
    drag: Option<Vector2<f64>>,

    cache:  BTreeMap<TilePos, Image>,
}

pub struct Image {
    size: Vector2<u32>,
    data: Vec<u8>,
}

impl State {
    pub fn init() -> Self {
        State {
            viewport: Viewport::new(),
            gpu: Gpu::init(),
            drag: None,
            cache: BTreeMap::new(),
        }
    }

    pub fn gen_tile(p: &TilePos) -> Image {
        let size = 256;
        let mut data = Vec::with_capacity(size as usize * size as usize * 4);

        let pos = p.square();

        let min = pos.corner_min();
        let max = pos.corner_max();

        for y in 0..size {
            for x in 0..size {
                let border = (y == 0 || y == size  - 1) || (x == 0 || x == size-1);

                let px = (x as f64 + 0.5) / size as f64;
                let py = (y as f64 + 0.5) / size as f64;

                let x = min.x as f64 *(1.0 - px) + max.x as f64 * px;
                let y = min.y as f64 *(1.0 - py) + max.y as f64 * py;

                let pi3 = std::f64::consts::FRAC_PI_3;
                let t = (x*x + y*y).sqrt()*5.0;


                let c = Vector2::new(x, y);
                let mut z = Vector2::new(0.0, 0.0);
                let mut t = 0.0;
                for i in 0..1024 {
                    z = Vector2::new(
                        z.x*z.x - z.y*z.y,
                        2.0*z.x*z.y
                    ) + c;

                    let d = z.x*z.x + z.y*z.y;
                    if d > 256.0 {
                        t += -d.log2().log2() + 4.0;
                        break;
                    }
                    t += 1.0;
                }

                let a = (1.0 - (t/(1024.0)).powi(2)).min(1.0).max(0.0);
                let t = t*0.1;
                let r = a * ((0.5 - t)*3.0*pi3 + pi3*0.0).sin();
                let g = a * ((0.5 - t)*3.0*pi3 + pi3*1.0).sin();
                let b = a * ((0.5 - t)*3.0*pi3 + pi3*2.0).sin();

                let r = r*r;
                let g = g*g;
                let b = b*b;

                data.push((r * 255.0) as _);
                data.push((g * 255.0) as _);
                data.push((b * 255.0) as _);
                data.push(255);
            }
        }

        Image { size: Vector2::new(size, size), data }
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

        for k in vp.get_pos_all() {
            if !self.cache.contains_key(&k) {
                let v = Self::gen_tile(&k);
                self.cache.insert(k, v);
                break;
            }
        }

        let cache = &self.cache;
        let tiles = vp.get_pos_all().filter_map(|p| cache.get(&p).map(|i| (p, i))).collect::<Vec<_>>();

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
        resolution: Vector2::new(resolution.width, resolution.height),
        mouse: Vector2::new(0, 0),
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

                    next_frame_time += Duration::from_secs_f32(target_dt);
                }

                *control_flow = ControlFlow::WaitUntil(next_frame_time);
            },
            _ => (),
        }
    });
}
