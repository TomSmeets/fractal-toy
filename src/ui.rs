use crate::AssetLoader;
use crate::Gpu;
use crate::Image;
use crate::util::*;

pub struct UI {
    pub mouse: V2,
    pub pos: V2,
    pub b: f64,
    pub w: f64,
}

impl UI {
    pub fn new() -> Self {
        UI {
            pos: V2::zero(),
            mouse: V2::zero(),
            b: 6.0,
            w: 80.0,
        }
    }

    pub fn mouse(&mut self, mouse: V2) {
        self.mouse = mouse;
    }

    pub fn begin(&mut self, res: V2) {
        self.pos.x = 0.0;
        self.pos.y = res.y - (self.b*2.0 + self.w)*2.0;
    }

    pub fn next_line(&mut self) {
        self.pos.x  = 0.0;
        self.pos.y += self.b*2.0 + self.w;
    }

    pub fn button(&mut self, gpu: &mut Gpu, asset: &mut AssetLoader, img: &Image) -> bool {
        let r = Rect::corner_size(self.pos + V2::new(self.b, self.b), V2::new(self.w, self.w));
        let ro = Rect::corner_size(self.pos, V2::new(self.w, self.w) + V2::new(self.b, self.b)*2.0);
        self.pos.x += self.w + self.b*2.0;

        gpu.blit(&r, img);

        if r.contains(self.mouse) {
            let img = asset.image("res/button_front_hot.png");
            gpu.blit(&r, &img);
        } else {
            let img = asset.image("res/button_front_norm.png");
            gpu.blit(&r, &img);
        }

        true
    }
}
