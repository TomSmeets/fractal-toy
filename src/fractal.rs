use crate::asset_loader::FontType;
use crate::builder::TileBuilder;
use crate::state::State;
use crate::update_loop::Input;
use crate::util::*;
use crate::viewport::Viewport;
use crate::viewport::ViewportInput;
use cgmath::{vec2, InnerSpace};
use winit::event::VirtualKeyCode;
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

pub struct Fractal {
    // actual state that is relevant
    builder: TileBuilder,
    viewport: Viewport,
    steps: Vec<FractalStep>,
}

impl Fractal {
    pub fn init(state: &mut State) -> Self {
        let steps = MANDELBROT.to_vec();
        let builder = TileBuilder::new(state.gpu.device(), &mut state.asset, &steps);
        Fractal {
            builder,
            viewport: Viewport::new(),
            steps,
        }
    }

    /// always called at regular intervals
    pub fn update(&mut self, state: &mut State, window: &Window, input: &Input) {
        let mut recreate_builder = false;

        state.debug.push("fractal.update()");

        {
            let mut dir: V2<f64> = vec2(0.0, 0.0);
            let mut speed = 1.0;
            let mut zoom = 0.0;
            for k in input.keys_down.iter() {
                match k {
                    VirtualKeyCode::W => dir.y += 1.0,
                    VirtualKeyCode::S => dir.y -= 1.0,
                    VirtualKeyCode::D => dir.x += 1.0,
                    VirtualKeyCode::A => dir.x -= 1.0,

                    VirtualKeyCode::Up => dir.y += 1.0,
                    VirtualKeyCode::Down => dir.y -= 1.0,
                    VirtualKeyCode::Right => dir.x += 1.0,
                    VirtualKeyCode::Left => dir.x -= 1.0,

                    VirtualKeyCode::LShift => speed = 3.0,
                    VirtualKeyCode::RShift => speed = 3.0,

                    VirtualKeyCode::I => zoom += 1.0,
                    VirtualKeyCode::K => zoom -= 1.0,
                    _ => (),
                }
            }
            let dir = dir / dir.magnitude().max(1.0) * speed * 1.0;

            let mut viewport_input = ViewportInput {
                dt: input.dt as f64,
                resolution: input.resolution,
                dir_move: dir,
                zoom_center: zoom * speed * 4.0,
                drag: None,
                scroll_at: (input.mouse, 0.0),
            };

            // handle input for the viewport, if the user didn't click the ui
            if !state.ui.has_input() {
                if input.mouse_down {
                    viewport_input.drag = Some(input.mouse);
                }
                viewport_input.scroll_at.1 = input.mouse_scroll as f64;
            }

            // resize viewport
            self.viewport.update(&viewport_input);
        }

        // queue which tiles should be built, we include a 1 tile border here
        state.debug.push("builder.tile() [build]");
        for p in self.viewport.get_pos_all(1) {
            self.builder.tile(&p);
        }
        state.debug.pop();

        // draw tiles, without a border, so just those visible
        state.debug.push("builder.tile() [draw]");
        for p in self.viewport.get_pos_all(0) {
            // if we don't have a tile don't draw it yet
            if let Some(img) = self.builder.tile(&p) {
                state.gpu.tile(&self.viewport, &p, img);
            }
        }
        state.debug.pop();

        // random information text
        state.debug.print(&Self::distance(self.viewport.scale));

        // The user interface buttons on the bottom
        {
            let window = state.ui.window("Buttons");
            state.debug.push("ui.buttons()");
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
            let size = vec2(100.0, 100.0);
            let mut pos = vec2(size.x * 0.5, self.viewport.size_in_pixels.y - size.y * 1.5);
            for s in STEP_VALUES.iter() {
                let rect = Rect::center_size(pos, size * 0.9);

                let image = state.asset.image(step_img(*s));
                let region = state.ui.region(&rect);

                let image_back = state.asset.image("res/button_back.png");
                state.gpu.blit(&rect, &image_back);
                state.gpu.blit(&rect, &image);

                let image_front = state.asset.image(if region.down {
                    "res/button_front_down.png"
                } else if region.hover {
                    "res/button_front_hot.png"
                } else {
                    "res/button_front_norm.png"
                });

                state.gpu.blit(&rect, &image_front);

                if region.click {
                    self.steps.push(*s);
                    recreate_builder = true;
                }

                pos.x += size.x;
            }
            pos.y += size.y;
            pos.x = size.x * 0.5;

            // and drop them here
            let mut remove = Vec::new();
            for (i, s) in self.steps.iter().enumerate() {
                let rect = Rect::center_size(pos, size * 0.9);

                let image = state.asset.image(step_img(*s));
                let region = state.ui.region(&rect);

                let image_back = state.asset.image("res/button_back.png");
                state.gpu.blit(&rect, &image_back);
                state.gpu.blit(&rect, &image);

                let image_front = state.asset.image(if region.down {
                    "res/button_front_down.png"
                } else if region.hover {
                    "res/button_front_hot.png"
                } else {
                    "res/button_front_norm.png"
                });

                state.gpu.blit(&rect, &image_front);

                if region.click {
                    remove.push(i);
                }

                pos.x += size.x;
            }

            for i in remove {
                self.steps.remove(i);
                recreate_builder = true;
            }

            state.debug.pop();
        }

        if true {
            state.debug.push("asset.text()");

            state.debug.push("asset.text(Cursor)");
            state.ui.window("Debug Text (Normal)").text(
                &mut state.asset,
                FontType::Mono,
                &state.debug.draw(),
            );
            state.debug.pop();

            state.debug.pop();
        }

        // update tile builder cache
        state.debug.push("builder.update()");
        self.builder.update();
        state.debug.pop();

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
            state.debug.print(&rate);
        }

        if recreate_builder {
            self.builder = TileBuilder::new(state.gpu.device(), &mut state.asset, &self.steps);
        }

        state.debug.pop();
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
}
