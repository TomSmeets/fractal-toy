use crate::asset_loader::AssetLoader;
use crate::asset_loader::FontType;
use crate::asset_loader::TextAlignment;
use crate::gpu::Gpu;
use crate::image::Image;
use crate::update_loop::Input;
use crate::util::*;
use std::collections::BTreeMap;

// Nested grid of centered rects
// then make as compact as possible
// each region has an id
//
// region: (id, parent_id, row, col)
//
// we can temporarly use row/col for id

type UIPos = V2<u16>;

// simple window, single grid/table layout
// future: many a list of tables, not nested, just linear
pub struct Window {
    name: &'static str,
    used: bool,

    position: V2,
    bounds: Rect,
    images: Vec<(Rect, Image)>,
}

impl Window {
    pub fn new(name: &'static str) -> Self {
        Window {
            name,
            used: true,
            bounds: Rect::min_max(V2::zero(), V2::zero()),
            position: V2::zero(),
            images: Vec::new(),
        }
    }

    pub fn content_rect(&self) -> Rect {
        Rect::corner_size(self.position, self.bounds.size())
    }

    pub fn unuse(&mut self) {
        self.used = false;
    }

    pub fn image(&mut self, rect: Rect, image: Image) {
        self.used = true;
        self.bounds.extend(&rect);
        self.images.push((rect, image));
    }

    pub fn text(&mut self, asset_loader: &mut AssetLoader, kind: FontType, text: &str) {
        let itr = asset_loader.text_iter(
            kind,
            V2::zero(),
            V2::new(TextAlignment::Left, TextAlignment::Left),
            26.0,
            text,
        );

        for (rect, img) in itr {
            self.image(rect, img);
        }
    }

    pub fn reset(&mut self) {
        self.bounds = Rect::min_max(V2::zero(), V2::zero());
        self.images.clear();
    }

    pub fn draw(&mut self, asset_loader: &mut AssetLoader, gpu: &mut Gpu) {
        let content_rect = self.content_rect();
        gpu.blit(&content_rect, &asset_loader.image("res/window_back.png"));
        for (rect, img) in self.images.iter() {
            let mut rect = *rect;
            rect.translate(-self.bounds.corner_min() + content_rect.corner_min());
            gpu.blit(&rect, img);
        }
    }

    pub fn button(&mut self, img: Image) -> bool {
        self.image(Rect::center_size(V2::zero(), V2::new(60.0, 60.0)), img);
        false
    }
}

// imgui: https://github.com/ocornut/imgui/blob/c881667c00655c98dba41deb942587e0041d0ed0/imgui_internal.h#L1410
pub struct UI {
    // Image is not that big, it uses Arc<>
    //
    // text is also just images. lest just snap those images to the cosest pixel
    // hinting is done when creating an entire string, but that string can be moved in pixel space.
    // hinting only makes sense wihtin a string.
    draggin_window: Option<(&'static str, V2)>,
    current_window: Option<Window>,
    windows: BTreeMap<&'static str, Window>,

    mouse: V2,
    mouse_down: bool,

    next_id: u32,
    hover: Option<u32>,
    hover_prev_frame: Option<u32>,
    down: Option<u32>,

    button_img: Image,
}

pub struct RegionResult {
    pub hover: bool,
    pub down: bool,
    pub click: bool,
}

impl UI {
    pub fn new(asset: &mut AssetLoader) -> Self {
        UI {
            next_id: 1,

            mouse: V2::zero(),
            mouse_down: false,

            current_window: None,
            draggin_window: None,
            windows: BTreeMap::new(),

            hover: None,
            hover_prev_frame: None,
            down: None,

            button_img: asset.image("res/button_front_hot.png"),
        }
    }

    pub fn region(&mut self, rect: &Rect) -> RegionResult {
        let id = self.next_id;
        self.next_id += 1;

        let mouse_in_rect = rect.contains(self.mouse);
        let mouse_down = self.mouse_down;

        let had_active = self.down.is_some();
        let was_active = self.down == Some(id);

        let hover = was_active || (!had_active && mouse_in_rect);
        let down = was_active || (!had_active && mouse_in_rect && mouse_down);
        let click = down && !was_active;

        if hover {
            self.hover = Some(id);
        }

        if click {
            self.down = Some(id);
        }

        RegionResult { hover, down, click }
    }

    pub fn has_input(&self) -> bool {
        let has_hover = self.hover.is_some() || self.hover_prev_frame.is_some();
        let has_down = self.down.is_some() && self.down != Some(0);
        has_hover || has_down || self.draggin_window.is_some()
    }

    pub fn next_row(&mut self) {}

    pub fn next_col(&mut self) {}

    pub fn window(&mut self, name: &'static str) -> &mut Window {
        let mut window = self.windows.entry(name).or_insert(Window::new(name));
        window.reset();
        window
    }

    pub fn update(&mut self, input: &Input, gpu: &mut Gpu, asset: &mut AssetLoader) {
        // end of frame
        if self.mouse_down && self.down.is_none() {
            self.down = Some(0)
        }
        self.hover_prev_frame = self.hover.take();

        // begin next frame
        self.mouse_down = input.mouse_down;
        self.mouse = input.mouse.map(|x| x as _);
        self.next_id = 1;

        if !self.mouse_down {
            self.down = None;
        }

        assert!(self.current_window.is_none());

        if self.draggin_window.is_some() && !input.mouse_down {
            self.draggin_window = None;
        }

        let mouse_pos = input.mouse.map(|x| x as _);
        let mut had_hover = false;
        for w in self.windows.values_mut() {
            let rect = w.content_rect();
            let hover = rect.contains(mouse_pos);

            if hover && input.mouse_down && self.draggin_window.is_none() {
                self.draggin_window = Some((w.name, mouse_pos - w.position));
            }

            if let Some((name, off)) = self.draggin_window {
                if name == w.name {
                    w.position = mouse_pos - off;

                    let mut rect = w.content_rect();
                    rect.grow(-16.0);

                    if rect.max.x < 0.0 {
                        w.position.x -= rect.max.x;
                    }
                    if rect.max.y < 0.0 {
                        w.position.y -= rect.max.y;
                    }

                    if rect.min.x > input.resolution.x as f64 {
                        w.position.x -= rect.min.x - input.resolution.x as f64;
                    }
                    if rect.min.y > input.resolution.y as f64 {
                        w.position.y -= rect.min.y - input.resolution.y as f64;
                    }
                }
            }

            w.draw(asset, gpu);
            w.reset();
            w.used = false;
        }
    }
}
