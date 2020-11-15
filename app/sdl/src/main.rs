mod atlas;
mod input;
mod main2;
mod sdl;

use fractal_toy::math::Rect;
use fractal_toy::Persist;
use fractal_toy::Reload;

pub fn rect_to_sdl(r: Rect) -> sdl2::rect::Rect {
    sdl2::rect::Rect::new(r.pos.x, r.pos.y, r.size.x as u32, r.size.y as u32)
}

fn main() {
    main2::run();
}
