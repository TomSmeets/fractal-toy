use crate::image::Image;
use crate::update_loop::Input;
use crate::util::*;
use crate::AssetLoader;
use crate::Gpu;

use crate::asset_loader::FontType;
use crate::asset_loader::TextAlignment;

// Nested grid of centered rects
// then make as compact as possible
// each region has an id
//
// region: (id, parent_id, row, col)
//
// we can temporarly use row/col for id

type UIPos = V2<u16>;

enum Command {
    NextRow,
    NextCol,
    MoveUp,
    MoveDown,
    Image(Rect, Image),
}

pub struct UI {
    current_pos: UIPos,
    parent_pos: Vec<UIPos>,

    // Image is not that big, it uses Arc<>
    //
    // text is also just images. lest just snap those images to the cosest pixel
    // hinting is done when creating an entire string, but that string can be moved in pixel space.
    // hinting only makes sense wihtin a string.
    elements: Vec<(UIPos, Image)>,

    commands: Vec<Command>,

    hover: Option<u32>,
    down: Option<u32>,
    click: Option<u32>,

    button_img: Image,
}

impl UI {
    pub fn new(asset: &mut AssetLoader) -> Self {
        UI {
            current_pos: V2::zero(),
            parent_pos: Vec::new(),
            elements: Vec::new(),
            commands: Vec::new(),
            hover: None,
            down: None,
            click: None,

            button_img: asset.image("res/button_front_hot.png"),
        }
    }

    pub fn has_input(&self) -> bool {
        self.hover.is_some() || self.down.is_some()
    }

    pub fn next_row(&mut self) {
        self.commands.push(Command::NextRow);
    }

    pub fn next_col(&mut self) {
        self.commands.push(Command::NextCol);
    }

    pub fn down(&mut self) {
        self.commands.push(Command::MoveDown);
    }

    pub fn up(&mut self) {
        self.commands.push(Command::MoveUp);
    }

    /// push an image, the rect is relative to the current cell
    pub fn image(&mut self, rect: Rect, img: Image) {
        self.commands.push(Command::Image(rect, img));
    }

    pub fn text(&mut self, asset_loader: &mut AssetLoader, text: &str) {
        let itr = asset_loader.text_iter(
            FontType::Normal,
            V2::zero(),
            V2::new(TextAlignment::Center, TextAlignment::Center),
            26.0,
            text,
        );

        for (rect, img) in itr {
            self.image(rect, img);
        }
        self.next_col();
    }

    pub fn button(&mut self, img: Image) -> bool {
        self.image(Rect::center_size(V2::zero(), V2::new(60.0, 60.0)), img);
        self.image(
            Rect::center_size(V2::zero(), V2::new(80.0, 80.0)),
            self.button_img.clone(),
        );
        self.next_col();
        false
    }

    pub fn update(&mut self, input: &Input, gpu: &mut Gpu, asset: &mut AssetLoader) {
        self.click = None;

        let pad = 8.0;

        let mut pos = V2 {
            x: 0.0,
            y: input.resolution.y as f64,
        };

        // current cell range
        // size of one tile
        let mut min = V2::<f64>::zero();
        let mut max = V2::<f64>::zero();
        let mut imgs = Vec::<(Rect, Image)>::new();
        for e in self.commands.iter() {
            match e {
                Command::NextCol => {
                    for (r, img) in imgs.drain(..) {
                        let off = V2::new(min.x, max.y);
                        let low = r.corner_min() - off + pos;
                        let high = r.corner_max() - off + pos;
                        let rect = Rect::min_max(low, high);
                        gpu.blit(&rect, &img);
                    }

                    pos.x += max.x - min.x;

                    min.x = 0.0;
                    max.x = 0.0;
                }

                Command::NextRow => {
                    pos.x = 0.0;
                    pos.y -= max.y - min.y;

                    min.y = 0.0;
                    max.y = 0.0;
                }

                Command::Image(rect, img) => {
                    min.x = min.x.min(rect.corner_min().x - pad);
                    min.y = min.y.min(rect.corner_min().y - pad);

                    max.x = max.x.max(rect.corner_max().x + pad);
                    max.y = max.y.max(rect.corner_max().y + pad);

                    imgs.push((*rect, img.clone()));
                }
                _ => {
                    // min = V2::zero();
                    // max = V2::zero();
                }
            }
        }

        self.commands.clear();

        /*
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
        */
    }
}
