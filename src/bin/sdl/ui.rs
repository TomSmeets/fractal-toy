use serial::fractal::Fractal;
use serial::math::Rect;
use serial::math::V2i;

#[derive(Clone, Copy)]
pub struct Input {
    pub viewport: V2i,
    pub mouse: V2i,
    pub left: bool,
    pub right: bool,
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

pub struct UI {
    pub input: Input,
    pub rects: Vec<(Rect, [u8; 3])>,

    pub active: u32,
}

impl UI {
    pub fn new() -> Self {
        UI {
            input: Input::new(),
            rects: Vec::new(),
            active: 0xffffffff,
        }
    }

    pub fn input(&mut self, input: Input) {
        self.input = input;
    }

    fn draw_rect(&mut self, rect: Rect, rgb: [u8; 3]) {
        let p = 2;
        let r2 = Rect::new(
            rect.pos.x - p,
            rect.pos.y - p,
            rect.size.x + 2 * p,
            rect.size.y + 2 * p,
        );
        self.rects.push((r2, [0, 0, 0]));
        self.rects.push((rect, rgb));
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

    fn button(&mut self, id: u32, rect: Rect) -> bool {
        let mut is_active = false;
        let mut is_hot = self.is_hot(rect);
        let mut went_down = false;

        if self.active == 0 {
            if is_hot && self.input.left {
                self.active = id;
                is_active = true;
                went_down = true;
            }
        } else if self.active == id {
            is_active = true;
        }

        let col_normal = [0, 0, 255];
        let col_hot = [0, 255, 0];
        let col_active = [255, 0, 0];

        let mut col = col_normal;

        if is_hot {
            col = col_hot;
        }

        if is_active {
            col = col_active;
        }

        self.draw_rect(rect, col);

        went_down
    }

    pub fn update<T>(&mut self, fractal: &mut Fractal<T>) {
        let zoom = fractal.pos.zoom;
        self.rects.clear();

        if !self.input.left && self.active != 0 {
            self.active = 0;
        }

        // draw slider
        {
            let w = 45;
            let pad = 10;
            let rect = Rect::new(
                self.input.viewport.x as i32 - w - pad,
                pad,
                w,
                self.input.viewport.y as i32 - pad * 2,
            );
            let slider_x = self.input.viewport.x as i32 - w / 2 - pad;
            {
                let rect = Rect::new(slider_x - 10, rect.pos.y, 20, rect.size.y);
                self.draw_rect(rect, [255, 255, 255]);
            }

            {
                let z = (zoom + 2.5) / (2.5 + 48.5);
                let z = z.max(0.0).min(1.0);
                let h = (z * rect.size.y as f64) as i32;
                let slider_radius = 10;
                let r_slider = Rect::new(
                    rect.pos.x,
                    rect.pos.y + h - slider_radius,
                    rect.size.x,
                    slider_radius * 2,
                );
                self.button(1, r_slider);
            }
        }

        // buttons
        {
            let mut x = 0;
            let w = 45;
            let pad = 10;

            for i in 0..6 {
                let rect = Rect::new(x + pad, self.input.viewport.y as i32 - w - pad, w, w);
                if self.button(i + 2, rect) {
                    println!("button: {}", i);
                }
                x += w + pad;
            }
        }
    }
}
