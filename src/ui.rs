use crate::AssetLoader;
use crate::Gpu;
use crate::Image;
use crate::util::*;

pub struct UI {
    pub mouse: Option<V2>,
    pub pos: V2,
    pub b: f64,
    pub w: f64,

    pub active_last_frame: bool,
    pub active:            bool,

    pub hot_element:    u32,
    pub active_element: u32,
}

impl UI {
    pub fn new() -> Self {
        UI {
            pos: V2::zero(),
            mouse: None,
            b: 6.0,
            w: 80.0,

            active: false,
            active_last_frame: false,

            hot_element:    0,
            active_element: 0,
        }
    }

    pub fn has_input(&self) -> bool {
        self.active_last_frame
    }

    pub fn mouse(&mut self, mouse: V2) {
        self.mouse = Some(mouse);
    }

    pub fn begin(&mut self, res: V2) {
        self.active_last_frame = self.active;
        self.active = false;
        self.pos.x = 0.0;
        self.pos.y = res.y - (self.b*2.0 + self.w)*2.0;
    }

    pub fn next_line(&mut self) {
        self.pos.x  = 0.0;
        self.pos.y += self.b*2.0 + self.w;
    }

    pub fn button(&mut self, gpu: &mut Gpu, asset: &mut AssetLoader, img: &Image) -> bool {
        let si = 5.0;
        let ri = Rect::corner_size(self.pos + V2::new(self.b + si, self.b + si), V2::new(self.w - si*2.0, self.w - si*2.0));
        let r  = Rect::corner_size(self.pos + V2::new(self.b, self.b), V2::new(self.w, self.w));
        let ro = Rect::corner_size(self.pos, V2::new(self.w, self.w) + V2::new(self.b, self.b)*2.0);
        self.pos.x += self.w + self.b*2.0;

        gpu.blit(&ri, img);

        match self.mouse {
            Some(mouse) if !self.active && r.contains(mouse) => {
                let img = asset.image("res/button_front_hot.png");
                gpu.blit(&r, &img);
                self.active = true;
            },
            _ => {
                let img = asset.image("res/button_front_norm.png");
                gpu.blit(&r, &img);
            }
        }
        true
    }
}
