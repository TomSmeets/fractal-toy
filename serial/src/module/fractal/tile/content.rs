use super::super::atlas::AtlasRegion;
use crate::module::fractal::gen::Gen;
use crate::module::fractal::TEXTURE_SIZE;

use super::pos::TilePos;
use sdl2::render::Texture;
use serde::{Deserialize, Serialize};

pub struct TileContent {
    pub pixels: Vec<u8>,
    pub region: Option<AtlasRegion>,
}

impl TileContent {
    pub fn new() -> TileContent {
        TileContent {
            pixels: Vec::new(),
            region: None,
        }
    }

    pub fn generate(&mut self, g: &Gen, p: TilePos) {
        self.pixels = g.generate(p);
    }

    pub fn to_sdl(&self, texture: &mut Texture) {
        if self.pixels.is_empty() {
            return;
        }

        texture
            .update(None, &self.pixels, (4 * TEXTURE_SIZE) as usize)
            .unwrap();
    }
}
