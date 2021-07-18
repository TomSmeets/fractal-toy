use crate::asset_loader::AssetLoader;
use crate::asset_loader::FontType;
use crate::debug::Debug;
use crate::gpu::Gpu;
use crate::ui::UI;
use crate::update_loop::Input;
use winit::window::Window;

pub struct State {
    pub gpu: Gpu,
    pub asset: AssetLoader,
    pub debug: Debug,
    pub ui: UI,
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
        }
    }

    pub fn update(&mut self, window: &Window, input: &Input) {
        self.debug.push("state.update()");

        self.debug.push("asset.text(Debug)");
        self.ui.window("Debug Text (mono)").text(
            &mut self.asset,
            FontType::Mono,
            &self.debug.draw(),
        );
        self.debug.pop();

        self.debug.push("ui.update()");
        self.ui.update(input, &mut self.gpu, &mut self.asset);
        self.debug.pop();

        // check for asset changes
        self.debug.push("asset.hot_reload()");
        self.asset.hot_reload();
        self.debug.pop();

        self.debug.push("gpu.update()");
        self.gpu.render(window, input.resolution, &mut self.debug);
        self.debug.pop();
        self.debug.pop();

        self.debug.begin();
    }
}
