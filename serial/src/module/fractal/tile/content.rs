use crate::module::fractal::{atlas::AtlasRegion, TEXTURE_SIZE};
use sdl2::render::Texture;

#[derive(Default)]
pub struct TileContent {
    pub pixels: Vec<u8>,
    pub region: Option<AtlasRegion>,
}

impl TileContent {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn to_sdl(&self, texture: &mut Texture) {
        if self.pixels.is_empty() {
            return;
        }

        texture
            .update(None, &self.pixels, 4 * TEXTURE_SIZE as usize)
            .unwrap();
    }
}
