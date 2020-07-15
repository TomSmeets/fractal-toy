// TODO: Arbirtrary precision, implementing arbitrary precision is not easy, but we probably want to use this: https://fractalwiki.org/wiki/Perturbation_theory
// TODO: osm tile builder
pub mod atlas;
mod colorscheme;
pub mod fractal;
pub mod input;
pub mod math;
pub mod state;
pub mod time;
pub mod ui;

pub use self::atlas::Atlas;
pub use self::colorscheme::ColorScheme;
pub use self::fractal::Fractal;
pub use self::input::Input;

pub use self::fractal::builder::IsTileBuilder;
pub use self::fractal::builder::TileParams;
pub use self::fractal::builder::TileType;
pub use self::fractal::content::TileContent;
pub use self::fractal::Task;
pub use tilemap::TilePos;
