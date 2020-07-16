use crate::math::Rect;
use crate::math::V2i;
use crate::Fractal;
use crate::TileType;

use ::ui::Id;
use ::ui::UIStack;

#[derive(Clone, Copy)]
pub struct UIInput {
    pub viewport: V2i,
    pub mouse: V2i,
    pub left: bool,
    pub right: bool,
}

impl UIInput {
    pub fn new() -> Self {
        UIInput {
            viewport: V2i::new(0, 0),
            mouse: V2i::new(0, 0),
            left: false,
            right: false,
        }
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct ButtonState {
    pub hot: bool,
    pub active: bool,
    pub click: bool,
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub enum Image {
    Fractal(TileType),
    Slider,
    ButtonFront(ButtonState),
    ButtonBack,
    Undefined,
}

// TODO: we should also have a reliable debug terminal interface
// It should contain debug counters and messages and commands to do everything
pub struct UI {
    pub input: UIInput,
    pub rects: Vec<(Rect, Image)>,

    active: Option<Id>,
    stack: UIStack,
}

impl UI {
    #[rustfmt::skip]
    pub fn to_path(img: Image) -> &'static [u8] {
        match img {
            Image::Fractal(TileType::Mandelbrot)  => include_bytes!("../../res/fractal_mandel.png"),
            Image::Fractal(TileType::ShipHybrid)  => include_bytes!("../../res/fractal_hybrid.png"),
            Image::Fractal(TileType::BurningShip) => include_bytes!("../../res/fractal_ship.png"),
            Image::Fractal(TileType::Empty)       => include_bytes!("../../res/fractal_missing.png"),

            Image::Slider => include_bytes!("../../res/slider.png"),

            Image::ButtonFront(ButtonState { active: true, .. }) => include_bytes!("../../res/button_front_down.png"),
            Image::ButtonFront(ButtonState { hot: true, .. })    => include_bytes!("../../res/button_front_hot.png"),
            Image::ButtonFront(ButtonState { .. })               => include_bytes!("../../res/button_front_norm.png"),

            Image::ButtonBack => include_bytes!("../../res/button_back.png"),
            _ => include_bytes!("../../res/missing.png"),
        }
    }

    pub fn new() -> Self {
        UI {
            input: UIInput::new(),
            rects: Vec::new(),
            active: None,
            stack: UIStack::default(),
        }
    }

    pub fn has_focus(&self) -> bool {
        self.active.is_some() && self.active != Some(Id::root())
    }

    pub fn input(&mut self, input: UIInput) {
        self.input = input;
    }

    fn draw_rect(&mut self, rect: Rect, img: Image) {
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

        self.draw_rect(rect, Image::ButtonBack);

        let state = ButtonState {
            hot: is_hot,
            active: is_active,
            click: went_down,
        };

        self.draw_rect(rect, Image::ButtonFront(state));
        self.stack.end();

        state
    }

    // TODO: urghh, i don't like that 'T' here
    pub fn update<T>(&mut self, fractal: &mut Fractal<T>) {
        // TODO: how should `hot` be handled with repsect to depth? just prevent overlap?
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
            self.draw_rect(rect, Image::Slider);
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
                (TileType::Mandelbrot, "Mandelbrot"),
                (TileType::BurningShip, "BurningShip"),
                (TileType::ShipHybrid, "ShipHybrid"),
                (TileType::Empty, "Empty"),
            ];
            for (t, name) in types.iter() {
                self.stack.begin(name);

                let rect = Rect::new(x + pad, self.input.viewport.y as i32 - w - pad, w, w);
                if self.button("button", rect).click {
                    fractal.params.kind = *t;
                    fractal.reload();
                }
                self.draw_rect(rect, Image::Fractal(*t));

                // TODO: pass this to the button call, don't render it here
                if fractal.params.kind == *t {
                    self.draw_rect(
                        rect,
                        Image::ButtonFront(ButtonState {
                            active: true,
                            hot: true,
                            click: false,
                        }),
                    );
                }

                x += w + pad;

                self.stack.end();
            }
        }

        if self.input.left && self.active == None {
            self.active = Some(Id::root());
        }
    }
}

impl Default for UI {
    fn default() -> Self {
        UI::new()
    }
}
