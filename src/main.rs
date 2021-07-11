// Sorry, but these warnings are very annoying
#![allow(dead_code)]
#![allow(unused_variables)]

mod asset_loader;
mod builder;
mod debug;
mod gpu;
mod image;
mod pack;
mod tilemap;
mod util;
mod viewport;
mod ui;
mod update_loop;

use self::asset_loader::AssetLoader;
use self::builder::TileBuilder;
use self::gpu::Gpu;
use self::image::Image;
use self::tilemap::TilePos;
use self::util::*;
use self::viewport::Viewport;
use self::ui::UI;
use self::update_loop::Loop;
use self::update_loop::Input;

use debug::Debug;
use std::collections::BTreeMap;
use std::process::Command;
use std::time::SystemTime;
use structopt::StructOpt;
use winit::window::Window;

static MANDELBROT: &[FractalStep] = &[FractalStep::Square, FractalStep::AddC];

static BURNINGSHIP: &[FractalStep] = &[
    FractalStep::AbsR,
    FractalStep::AbsI,
    FractalStep::Square,
    FractalStep::AddC,
];

static SNAIL: &[FractalStep] = &[
    // Mandelbrot
    FractalStep::Square,
    FractalStep::AddC,
    // ship
    FractalStep::AbsI,
    FractalStep::AbsR,
    FractalStep::Square,
    FractalStep::AddC,
];

static COOL: &[FractalStep] = &[
    FractalStep::AbsI,
    FractalStep::AbsR,
    FractalStep::Square,
    FractalStep::AddC,

    FractalStep::Square,
    FractalStep::AddC,

    FractalStep::Square,
    FractalStep::AddC,

    FractalStep::Square,
    FractalStep::AddC,
];

static STEP_VALUES: &[FractalStep] = &[
    FractalStep::Square,
    FractalStep::Cube,
    FractalStep::AddC,
    FractalStep::AbsR,
    FractalStep::AbsI,
];

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum FractalStep {
    /// z = z^2
    Square,

    /// z = z^3
    Cube,

    /// z = |real(z)| + imag(z) * i
    AbsR,

    /// z = real(z) - |imag(z)| * i
    AbsI,

    ///  z = z + c
    AddC,
}

#[derive(Debug, StructOpt)]
struct Config {
    #[structopt(short, long)]
    move_window: Option<Option<u32>>,

    #[structopt(short, long)]
    debug: bool,
}

pub struct State {
    gpu: Gpu,
    builder: TileBuilder,
    asset: AssetLoader,
    debug: Debug,
    ui: UI,

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
            ui: UI::new(),
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
    pub fn update(&mut self, window: &Window, input: &Input) {
        self.debug.time("viewport");
        // viewport stuff
        self.viewport.size(input.resolution);

        self.ui.begin(self.viewport.size_in_pixels);
        self.ui.mouse(input.mouse.map(|x| x as _), input.mouse_down);
        self.debug.print(&format!("{:?}", self.ui.active_element));

        if !self.ui.has_input() {
            self.viewport.zoom_at(input.mouse_scroll as f64, input.mouse);

            if input.mouse_down {
                self.viewport.drag(input.mouse);
            }
        }

        self.viewport.update(input.dt as f64);



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


        {
            fn step_img(s: FractalStep) -> &'static str{
                match s {
                    FractalStep::Square => "res/mod_2.png",
                    FractalStep::Cube => "res/mod_3.png",
                    FractalStep::AbsR => "res/mod_abs_r.png",
                    FractalStep::AbsI => "res/mod_abs_i.png",
                    FractalStep::AddC => "res/mod_c.png",
                }
            }


            /*
            viewport: [pos]
            builder:  pos -> Option<ImageID>
            gpu:      ImageID -> ()
            */
            
            // Pick modules from these
            for (i, s) in STEP_VALUES.iter().enumerate() {
                let img = self.asset.data(step_img(*s));
                let img = self.asset.image(img);
                self.ui.button(&mut self.gpu, &mut self.asset, img);
            }

            self.ui.next_line();

            // and drop them here
            for (i, s) in COOL.iter().enumerate() {
                let img = self.asset.data(step_img(*s));
                let img = self.asset.image(img);
                self.ui.button(&mut self.gpu, &mut self.asset, img);
            }
        }

        self.debug.time("gpu render");
        self.gpu.render(window, &self.viewport, &mut self.debug);

        self.debug.time("builder update");
        self.builder.update();

        self.asset.hot_reload();
    }
}

/*
pub fn test() {
    Window::run(|| {
        w.title("Fractal Toy");

        w.input
        
    })
}
*/



pub fn main() {
    let config = Config::from_args();
    let update_loop = Loop::new();

    if let Some(ws) = config.move_window {
        let ws = ws.unwrap_or(9);

        use winit::platform::unix::WindowExtUnix;
        // very hacky way to move the window out of my way
        // when using 'cargo watch -x run'
        // was to lazy to modify my wm or so.
        // This actually works very well :)
        if let Some(id) = update_loop.window.xlib_window() {
            let _ = Command::new("wmctrl")
                .arg("-i")
                .arg("-r")
                .arg(id.to_string())
                .arg("-t")
                .arg(ws.to_string())
                .status();
        }
    }

    let mut state = State::init(&update_loop.window);

    update_loop.run(move |window, input| {
        state.debug.begin();
        state.debug.time("state.update (start)");
        state.update(&window, &input);
        state.debug.time("state.update (end)");

        /*
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
        */
    });
}
