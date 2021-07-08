use crate::AssetLoader;
use crate::Gpu;
use crate::Image;
use crate::util::*;

pub struct UI {
    pub mouse: Option<V2>,
    pub mouse_down: bool,

    pub pos: V2,
    pub b: f64,
    pub w: f64,

    pub id: u32,

    pub hot_element_last_frame:    Option<u32>,
    pub hot_element:    Option<u32>,
    pub active_element: Option<u32>,
}

impl UI {
    pub fn new() -> Self {
        UI {
            pos: V2::zero(),
            mouse: None,
            mouse_down: false,
            b: 6.0,
            w: 80.0,
            id: 0,

            hot_element:    None,
            active_element: None,
            hot_element_last_frame:    None,
        }
    }

    pub fn has_input(&self) -> bool {
        self.hot_element.is_some() || self.hot_element_last_frame.is_some() || self.active_element.is_some()
    }

    pub fn mouse(&mut self, mouse: V2, down: bool) {
        self.mouse = Some(mouse);
        self.mouse_down = down;
        if !self.mouse_down { self.active_element = None; }
    }

    pub fn begin(&mut self, res: V2) {
        self.hot_element_last_frame = self.hot_element.take();
        self.pos.x = 0.0;
        self.pos.y = res.y - (self.b*2.0 + self.w)*2.0;
        self.id = 0;
    }

    pub fn next_line(&mut self) {
        self.pos.x  = 0.0;
        self.pos.y += self.b*2.0 + self.w;
    }

    pub fn button(&mut self, gpu: &mut Gpu, asset: &mut AssetLoader, img: &Image) -> bool {
        let si = 5.0;
        let mut ri = Rect::corner_size(self.pos + V2::new(self.b + si, self.b + si), V2::new(self.w - si*2.0, self.w - si*2.0));
        let r  = Rect::corner_size(self.pos + V2::new(self.b, self.b), V2::new(self.w, self.w));
        let ro = Rect::corner_size(self.pos, V2::new(self.w, self.w) + V2::new(self.b, self.b)*2.0);


        let is_hover = match self.mouse {
            Some(m) => r.contains(m),
            None    => false,
        };

        let is_hot = is_hover && self.hot_element.is_none();

        let is_active = match self.active_element {
            Some(id) => self.id == id,
            None     => self.mouse_down && is_hot,
        };

        if is_active {
            ri = Rect::center_size(self.mouse.unwrap(), V2::new(1.0, 1.0)*self.w);
        }

        gpu.blit(&ri, img);

        if is_active {
            gpu.blit(&r, &asset.image("res/button_front_down.png"));
            self.active_element = Some(self.id);
        } else if is_hot {
            gpu.blit(&r, &asset.image("res/button_front_hot.png"));
            self.hot_element = Some(self.id);
        } else {
            gpu.blit(&r, &asset.image("res/button_front_norm.png"));
        }


        self.id += 1;
        self.pos.x += self.w + self.b*2.0;

        is_active
    }
}
