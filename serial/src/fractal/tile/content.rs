use crate::fractal::gen::Gen;
use crate::fractal::TEXTURE_SIZE;

use super::pos::TilePos;
use sdl2::render::Texture;

pub struct TileContent {
    pub old: bool,     // did this tile become obsolete
    pub dirty: bool,   // does this tiles till need to be generated
    pub working: bool, // is a thread working on this tile
    pixels: Vec<u8>,
}

impl TileContent {
    pub fn new() -> TileContent {
        TileContent {
            old: false,
            dirty: true,
            working: false,
            pixels: Vec::new(),
        }
    }

    pub fn generate(&mut self, g: &Gen, p: TilePos) {
        let pixels = g.generate(p);
        self.dirty = false;
        self.pixels = pixels;
        self.working = false;
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
