use serial::fractal::Fractal;
use serial::math::Rect;
use serial::math::V2i;

pub struct Input {
    pub viewport: V2i,
    pub mouse: V2i,
    pub left: bool,
    pub right: bool,
}

#[derive(Default)]
pub struct UI {
    pub rects: Vec<(Rect, [u8; 3])>,
}

impl UI {
    pub fn new() -> Self {
        Self::default()
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

    fn button(&mut self, rect: Rect) -> bool {
        let is_active = false;
        let is_hot = false;
        let col_nor = [0, 0, 255];
        let col_hot = [0, 255, 0];
        let col_act = [255, 0, 0];

        false
    }

    pub fn update(&mut self, input: &Input, zoom: f64) {
        self.rects.clear();

        // draw slider
        {
            let w = 45;
            let pad = 10;
            let rect = Rect::new(
                input.viewport.x as i32 - w - pad,
                pad,
                w,
                input.viewport.y as i32 - pad * 2,
            );
            let slider_x = input.viewport.x as i32 - w / 2 - pad;
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
                self.draw_rect(r_slider, [255, 0, 0]);
            }
        }

        // buttons
        {
            let mut x = 0;
            let w = 45;
            let pad = 10;

            for _ in 0..6 {
                let rect = Rect::new(x + pad, input.viewport.y as i32 - w - pad, w, w);
                self.draw_rect(rect, [255, 0, 0]);
                x += w + pad;
            }
        }
    }
}
