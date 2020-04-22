// TODO: Arbirtrary precision
pub mod atlas;
pub mod iter;
pub mod math;
pub mod state;

pub mod fractal;
pub mod input;
pub mod time;

pub use self::{atlas::Atlas, fractal::Fractal, input::Input, time::Time};
