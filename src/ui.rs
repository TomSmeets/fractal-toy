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
    pub fn button(&mut self, gpu: &mut Gpu, asset: &mut AssetLoader, img: &Image) -> bool {
        let r = Rect::corner_size(self.pos + V2::new(self.b, self.b), V2::new(self.w, self.w));
        let ro = Rect::corner_size(self.pos, V2::new(self.w, self.w) + V2::new(self.b, self.b)*2.0);
        self.pos.x += self.w + self.b*2.0;

        gpu.blit(&r, img);

        if ro.contains(self.mouse) {
            let img = asset.image("res/button_front_hot.png");
            gpu.blit(&r, &img);
        } else {
            let img = asset.image("res/button_front_norm.png");
            gpu.blit(&r, &img);
        }

        true
    }
}
