use crate::fractal::gen::Gen;
use crate::fractal::TEXTURE_SIZE;

use super::pos::TilePos;
use sdl2::render::Texture;

pub struct TileContent {
    pixels: Vec<u8>,
}

impl TileContent {
    pub fn new(p: TilePos) -> TileContent {
        let pixels = Gen::generate(p);
        TileContent { pixels }
    }

    pub fn to_sdl(&self, texture: &mut Texture) {
        texture
            .update(None, &self.pixels, (4 * TEXTURE_SIZE) as usize)
            .unwrap();
    }
}
