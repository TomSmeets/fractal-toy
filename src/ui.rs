use crate::image::Image;
use crate::update_loop::Input;
use crate::util::*;
use crate::AssetLoader;
use crate::Gpu;

pub struct UI {
    row: u32,
    col: u32,

    // Image is not that big, it uses Arc<>
    elements: Vec<(u32, u32, Image)>,

    hover: Option<u32>,
    down: Option<u32>,
    click: Option<u32>,
}

impl UI {
    pub fn new() -> Self {
        UI {
            row: 0,
            col: 0,
            elements: Vec::new(),
            hover: None,
            down: None,
            click: None,
        }
    }

    pub fn has_input(&self) -> bool {
        self.hover.is_some() || self.down.is_some()
    }

    pub fn next_line(&mut self) {
        self.row += 1;
        self.col = 0;
    }

    pub fn button(&mut self, img: Image) -> bool {
        let id = self.elements.len() as _;
        self.elements.push((self.row, self.col, img));
        self.col += 1;
        self.click == Some(id)
    }

    pub fn update(&mut self, input: &Input, gpu: &mut Gpu, asset: &mut AssetLoader) {
        self.click = None;

        let size_border = 6;
        let size_inner = 80;
        let size_outer = size_inner + size_border * 2;

        let line_count = self.row + 1;

        let mut hover = None;
        let mut rects = Vec::new();
        for (id, (row, col, img)) in self.elements.drain(..).enumerate() {
            let id = id as u32;

            let x = col * size_outer;
            let y = input.resolution.y - (line_count - row) * size_outer;

            let inner_min = V2::new(x + size_border, y + size_border);
            let inner_max = inner_min + V2::new(size_inner, size_inner);
            let inner = Rect::min_max(inner_min.map(|x| x as _), inner_max.map(|x| x as _));

            let is_hover = inner.contains(input.mouse.map(|x| x as _));

            if is_hover {
                hover = Some(id);
            }

            rects.push((id, inner, img));
        }

        if !input.mouse_down && self.down.is_some() {
            self.down = None;
        }

        if input.mouse_down && self.down.is_none() {
            self.down = hover;
            self.click = hover;
        }

        for (id, rect, img) in rects.drain(..) {
            let id = id as u32;

            let is_hover = hover == Some(id);
            let is_active = self.down == Some(id);

            // back
            gpu.blit(&rect, &asset.image("res/button_back.png"));

            // image
            gpu.blit(&rect, &img);

            // front
            {
                let id = if is_active {
                    asset.image("res/button_front_down.png")
                } else if is_hover {
                    asset.image("res/button_front_hot.png")
                } else {
                    asset.image("res/button_front_norm.png")
                };
                gpu.blit(&rect, &id);
            }
        }

        self.row = 0;
        self.col = 0;
    }
}
