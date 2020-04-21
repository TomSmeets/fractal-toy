pub mod fractal;
pub mod input;
pub mod time;

#[cfg(feature = "platform-sdl")]
pub mod sdl;

#[cfg(feature = "platform-sdl")]
pub use self::sdl::Sdl;

pub use self::{fractal::Fractal, input::Input, time::Time};
