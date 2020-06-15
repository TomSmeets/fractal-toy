// TODO: Arbirtrary precision, implementing arbitrary precision is not easy, but we probably want to use this: https://fractalwiki.org/wiki/Perturbation_theory
pub mod atlas;
pub mod math;
pub mod state;

pub mod fractal;
pub mod input;
pub mod time;

pub mod tilemap;
pub mod util;

pub use self::{atlas::Atlas, fractal::Fractal, input::Input};

mod colorscheme;
pub use colorscheme::ColorScheme;
