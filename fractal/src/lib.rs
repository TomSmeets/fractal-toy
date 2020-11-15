// TODO: Arbirtrary precision, implementing arbitrary precision is not easy, but we probably want to use this: https://fractalwiki.org/wiki/Perturbation_theory
// TODO: osm tile builder
// TODO: only export a few types to simplify the api
// TODO: wgpu backend
mod atlas;
mod colorscheme;
mod fractal;
mod input;
pub mod math;
mod state;
mod time;
mod ui;

pub use self::atlas::Atlas;
pub use self::atlas::AtlasRegion;
pub use self::atlas::AtlasTextureProvider;
pub use self::atlas::SimpleAtlas;
pub use self::colorscheme::ColorScheme;
pub use self::fractal::builder::IsTileBuilder;
pub use self::fractal::builder::TileParams;
pub use self::fractal::builder::TileType;
pub use self::fractal::content::TileContent;
pub use self::fractal::viewport::Viewport;
pub use self::fractal::Fractal;
pub use self::fractal::FractalSave;
pub use self::fractal::Task;
pub use self::fractal::TileTextureProvider;
pub use self::fractal::TEXTURE_SIZE;
pub use self::input::Input;
pub use self::input::InputAction;
pub use self::input::InputEvent;
pub use self::math::*;
pub use self::state::Persist;
pub use self::state::Reload;
pub use self::time::DeltaTime;
pub use self::ui::Image;
pub use self::ui::UIInput;
pub use self::ui::UI;
pub use tilemap::TilePos;
