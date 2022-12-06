use cgmath::vec2;
use rusttype::Scale;
use winit::window::Window;

use crate::asset_loader::AssetLoader;
use crate::asset_loader::FontType;
use crate::asset_loader::TextAlignment;
use crate::debug::Debug;
use crate::gpu::Gpu;
use crate::ui::UI;
use crate::update_loop::Input;
use crate::util::Rect;

pub struct State {
    pub gpu: Gpu,
    pub asset: AssetLoader,
    pub debug: Debug,
    pub ui: UI,

    show_debug: bool,
}

impl State {
    pub fn init(window: &Window) -> Self {
        let mut asset = AssetLoader::new();
        let gpu = Gpu::init(window, &mut asset);
        let ui = UI::new(&mut asset);
        State {
            debug: Debug::new(),
            gpu,
            asset,
            ui,
            show_debug: false,
        }
    }

    pub fn update(&mut self, window: &Window, input: &Input) {
        Debug::push("state.update()");

        if input.key_click(winit::event::VirtualKeyCode::Key1) {
            self.show_debug = !self.show_debug;
        }

        if self.show_debug {
            Debug::push("asset.text(Debug)");
            let font_type = FontType::Mono;
            let font_size = 26.0;
            let text = self.debug.draw();

            let bounds = self
                .asset
                .text_bounds(font_type, Scale::uniform(font_size), &text);
            let mut rect = Rect::corner_size(vec2(0.0, 0.0), bounds.size()); // Rect::corner_size(vec2(0.0, 0.0), vec2(400.0, 400.0));
            rect.translate(vec2(100.0, 100.0));
            let region = self.ui.region(&rect);
            let image = self.asset.image("window_back.png");
            self.gpu.blit(&rect, &image);

            self.asset.text(
                FontType::Mono,
                rect.corner_min().map(|x| x as _),
                vec2(TextAlignment::Left, TextAlignment::Left),
                26.0,
                &mut self.gpu,
                &text,
            );
            Debug::pop();
        }

        Debug::push("ui.update()");
        self.ui.update(input, &mut self.gpu, &mut self.asset);
        Debug::pop();

        // check for asset changes
        Debug::push("asset.hot_reload()");
        self.asset.hot_reload();
        Debug::pop();

        Debug::push("gpu.update()");
        self.gpu.render(window, input.resolution);
        Debug::pop();
        Debug::pop();

        self.debug.begin();
    }
}
