use crate::asset_loader::AssetLoader;
use crate::asset_loader::FontType;
use crate::builder::TileBuilder;
use crate::debug::Debug;
use crate::gpu::Gpu;
use crate::ui::UI;
use crate::update_loop::Input;
use crate::viewport::Viewport;
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
    FractalStep::Conj,
];

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
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

    /// complex conjugate
    Conj,
}

pub struct State {
    gpu: Gpu,
    builder: TileBuilder,
    asset: AssetLoader,
    debug: Debug,
    ui: UI,

    // actual state that is relevant
    viewport: Viewport,

    steps: Vec<FractalStep>,
}

impl State {
    pub fn init(window: &Window) -> Self {
        let mut asset = AssetLoader::new();
        let gpu = Gpu::init(window, &mut asset);
        let steps = MANDELBROT.to_vec();
        let builder = TileBuilder::new(gpu.device(), &mut asset, &steps);
        let ui = UI::new(&mut asset);
        State {
            debug: Debug::new(),
            gpu,
            builder,
            viewport: Viewport::new(),
            asset,
            ui,
            steps,
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
        let mut recreate_builder = false;

        self.debug.begin();
        self.debug.push("update()");

        // resize viewport
        self.viewport.size(input.resolution);

        // handle input for the viewport, if the user didn't click the ui
        if !self.ui.has_input() {
            self.viewport
                .zoom_at(input.mouse_scroll as f64, input.mouse);

            if input.mouse_down {
                self.viewport.drag(input.mouse);
            }
        }

        // animate the viewport
        self.viewport.update(input.dt as f64);

        // check for asset changes
        self.debug.push("asset.hot_reload()");
        self.asset.hot_reload();
        self.debug.pop();

        // queue which tiles should be built, we include a 1 tile border here
        self.debug.push("builder.tile() [build]");
        for p in self.viewport.get_pos_all(1) {
            self.builder.tile(&p);
        }
        self.debug.pop();

        // draw tiles, without a border, so just those visible
        self.debug.push("builder.tile() [draw]");
        for p in self.viewport.get_pos_all(0) {
            // if we don't have a tile don't draw it yet
            if let Some(img) = self.builder.tile(&p) {
                self.gpu.tile(&self.viewport, &p, img);
            }
        }
        self.debug.pop();

        // random information text
        self.debug.print(&Self::distance(self.viewport.scale));

        // The user interface buttons on the bottom
        {
            let window = self.ui.window("Buttons");
            self.debug.push("ui.buttons()");
            fn step_img(s: FractalStep) -> &'static str {
                match s {
                    FractalStep::Square => "res/mod_2.png",
                    FractalStep::Cube => "res/mod_3.png",
                    FractalStep::AbsR => "res/mod_abs_r.png",
                    FractalStep::AbsI => "res/mod_abs_i.png",
                    FractalStep::AddC => "res/mod_c.png",
                    FractalStep::Conj => "res/mod_conj.png",
                }
            }

            // self.ui.text(&mut self.asset, &self.debug.draw());

            // Pick modules from these
            for s in STEP_VALUES.iter() {
                window.text(&mut self.asset, FontType::Mono, &format!("{:?}", s));
                if window.button(self.asset.image(step_img(*s))) {
                    self.steps.push(*s);
                    recreate_builder = true;
                }
            }

            // and drop them here
            let window = self.ui.window("Current Steps");
            let mut remove = Vec::new();
            for (i, s) in self.steps.iter().enumerate() {
                if window.button(self.asset.image(step_img(*s))) {
                    remove.push(i);
                }
            }

            for i in remove {
                self.steps.remove(i);
                recreate_builder = true;
            }

            self.debug.pop();
        }

        if true {
            self.debug.push("asset.text()");

            self.debug.push("asset.text(Debug)");
            self.ui.window("Debug Text (mono)").text(
                &mut self.asset,
                FontType::Mono,
                &self.debug.draw(),
            );
            self.debug.pop();

            self.debug.push("asset.text(Cursor)");
            self.ui.window("Debug Text (Normal)").text(
                &mut self.asset,
                FontType::Mono,
                &self.debug.draw(),
            );
            self.debug.pop();

            self.debug.pop();
        }

        self.debug.push("ui.update()");
        self.ui.update(input, &mut self.gpu, &mut self.asset);
        self.debug.pop();

        self.debug.push("gpu.update()");
        self.gpu.render(window, &self.viewport, &mut self.debug);
        self.debug.pop();

        // update tile builder cache
        self.debug.push("builder.update()");
        self.builder.update();
        self.debug.pop();

        {
            let dt_frame = input.real_dt_full;
            let dt_update = input.real_dt_update;
            let rate = format!(
                "real {:6.1} Hz ({:6} µs)\nbest {:6.1} Hz ({:6} µs)",
                1.0 / dt_frame.as_secs_f32(),
                dt_frame.as_micros(),
                1.0 / dt_update.as_secs_f32(),
                dt_update.as_micros(),
            );
            self.debug.print(&rate);
        }

        if recreate_builder {
            self.builder = TileBuilder::new(self.gpu.device(), &mut self.asset, &self.steps);
        }

        self.debug.pop();
    }
}
