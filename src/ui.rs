use crate::fractal::Fractal;
use crate::fractal::TileType;
use crate::math::Rect;
use crate::math::V2i;

use ::ui::Id;
use ::ui::UIStack;

#[derive(Clone, Copy)]
pub struct Input {
    pub viewport: V2i,
    pub mouse: V2i,
    pub left: bool,
    pub right: bool,
}

pub struct ButtonState {
    pub hot: bool,
    pub active: bool,
    pub click: bool,
}

impl Input {
    pub fn new() -> Self {
        Input {
            viewport: V2i::new(0, 0),
            mouse: V2i::new(0, 0),
            left: false,
            right: false,
        }
    }
}

pub enum Image {
    Rect,
    ButtonBack,
    ButtonFront { down: bool, hot: bool },
}

pub struct UI {
    pub input: Input,
    pub rects: Vec<(Rect, &'static str)>,

    active: Option<Id>,
    stack: UIStack,
}

impl UI {
    pub fn new() -> Self {
        UI {
            input: Input::new(),
            rects: Vec::new(),
            active: None,
            stack: UIStack::default(),
        }
    }

    pub fn has_focus(&self) -> bool {
        self.active.is_some()
    }

    pub fn input(&mut self, input: Input) {
        self.input = input;
    }

    fn draw_rect(&mut self, rect: Rect, img: &'static str) {
        self.rects.push((rect, img));
    }

    // hot is hover, but only the top most
    fn is_hot(&self, rect: Rect) -> bool {
        let lx = rect.pos.x;
        let ly = rect.pos.y;
        let hx = rect.pos.x + rect.size.x;
        let hy = rect.pos.y + rect.size.y;

        let mx = self.input.mouse.x;
        let my = self.input.mouse.y;

        let in_x = mx > lx && mx < hx;
        let in_y = my > ly && my < hy;
        in_x && in_y
    }

    fn button(&mut self, name: &str, rect: Rect) -> ButtonState {
        let id = self.stack.begin(name);

        let mut is_active = false;
        let is_hot = self.is_hot(rect);
        let mut went_down = false;

        if self.active.is_none() {
            if is_hot && self.input.left {
                self.active = Some(id);
                is_active = true;
                went_down = true;
            }
        } else if self.active == Some(id) {
            is_active = true;
        }

        self.draw_rect(rect, "button_back");

        if is_active {
            self.draw_rect(rect, "button_front_down");
        } else if is_hot {
            self.draw_rect(rect, "button_front_hot");
        } else {
            self.draw_rect(rect, "button_front_norm");
        }

        self.stack.end();

        ButtonState {
            hot: is_hot,
            active: is_active,
            click: went_down,
        }
    }

    // TODO: urghh, i don't like that 'T' here
    pub fn update<T>(&mut self, fractal: &mut Fractal<T>) {
        // TODO: how should `hot` be handled with repsect to depth?
        // if self.input.left && self.active.is_none() {
        //     self.active = Some(Id::new("BG", Id::root()));
        // }

        // begin
        let zoom = fractal.pos.zoom;
        self.rects.clear();

        if !self.input.left && self.active.is_some() {
            self.active = None;
        }

        let ui_size = 64;
        // draw slider
        {
            let w = ui_size;
            let pad = 64;
            let rect = Rect::new(
                self.input.viewport.x as i32 - w - pad,
                pad,
                w,
                self.input.viewport.y as i32 - pad * 2,
            );
            self.draw_rect(rect, "slider");
            {
                let z = (zoom + 2.5) / (2.5 + 48.5);
                let z = z.max(0.0).min(1.0);
                let h = (z * rect.size.y as f64) as i32;
                let r_slider =
                    Rect::new(rect.pos.x, rect.pos.y + h - ui_size / 2, ui_size, ui_size);

                if self.button("zoom-slider", r_slider).active {
                    let z = self.input.mouse.y - rect.pos.y;
                    let z = z as f64 / rect.size.y as f64;
                    let z = z.max(0.0).min(1.0);
                    let z = z * (2.5 + 48.5) - 2.5;
                    fractal.pos.zoom = z;
                }
            }
        }

        // buttons
        {
            let mut x = 0;
            let w = 64;
            let pad = 10;

            let types = [
                (TileType::Mandelbrot, "Mandelbrot", "fractal_mandel"),
                (TileType::BurningShip, "BurningShip", "fractal_ship"),
                (TileType::ShipHybrid, "ShipHybrid", "fractal_hybrid"),
                (TileType::Empty, "Empty", "fractal_missing"),
            ];
            for (t, name, img) in types.iter() {
                self.stack.begin(name);

                let rect = Rect::new(x + pad, self.input.viewport.y as i32 - w - pad, w, w);
                if self.button("button", rect).click {
                    fractal.params.kind = *t;
                    fractal.reload();
                }
                self.draw_rect(rect, img);
                if fractal.params.kind == *t {
                    self.draw_rect(rect, "button_front_down");
                }
                x += w + pad;

                self.stack.end();
            }
        }
    }
}
