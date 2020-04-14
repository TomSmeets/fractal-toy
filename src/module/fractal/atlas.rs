use super::TileTextureProvider;
use crate::math::*;
use crate::module::Sdl;
use sdl2::render::Texture;
use serde::{Deserialize, Serialize};

pub const PADDING: u32 = 1;

pub struct Atlas {
    pub size: u32,
    pub res: u32,
    pub free: Vec<Vector3<u32>>,
    pub texture: Vec<Texture>,
}

impl Atlas {
    pub fn new() -> Atlas {
        let res = super::TEXTURE_SIZE as u32;
        let size = 64 * 64 / res;
        Atlas {
            free: Vec::new(),
            size,
            res,
            texture: Vec::new(),
        }
    }

    pub fn alloc_page(&mut self, sdl: &mut Sdl) {
        let page = self.texture.len();

        let texture = sdl.create_texture_static_rgba8(self.size * self.res, self.size * self.res);

        self.texture.push(texture);

        self.free.reserve(self.size as usize * self.size as usize);
        for j in 0..self.size {
            for i in 0..self.size {
                self.free.push(Vector3::new(i, j, page as u32));
            }
        }
    }

    pub fn alloc(&mut self, sdl: &mut Sdl) -> AtlasRegion {
        match self.free.pop() {
            Some(i) => AtlasRegion {
                index: i,
                res: self.res,
                free: false,
            },
            None => {
                self.alloc_page(sdl);
                self.alloc(sdl)
            },
        }
    }

    pub fn update(&mut self, r: &AtlasRegion, pixels: &[u8]) {
        let r1 = r.rect();
        let t = &mut self.texture[r.index.z as usize];
        t.update(Some(r1.to_sdl()), pixels, 4 * self.res as usize)
            .unwrap();
    }

    pub fn remove(&mut self, mut r: AtlasRegion) {
        self.free.push(r.index);
        r.free = true;
    }
}

impl Default for Atlas {
    fn default() -> Self {
        Atlas::new()
    }
}

impl Drop for Atlas {
    fn drop(&mut self) {
        for t in self.texture.drain(..) {
            unsafe {
                t.destroy();
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AtlasRegion {
    pub index: Vector3<u32>,
    res: u32,
    free: bool,
}

impl AtlasRegion {
    pub fn rect(&self) -> Rect {
        let pos = Vector2 {
            x: (self.res * self.index.x) as i32,
            y: (self.res * self.index.y) as i32,
        };

        let size = Vector2 {
            x: (self.res) as i32,
            y: (self.res) as i32,
        };

        Rect { pos, size }
    }

    // returns smaller rectangle with padding removed
    pub fn rect_padded(&self) -> Rect {
        let pos = Vector2 {
            x: (self.res * self.index.x + PADDING) as i32,
            y: (self.res * self.index.y + PADDING) as i32,
        };

        let size = Vector2 {
            x: (self.res - PADDING * 2) as i32,
            y: (self.res - PADDING * 2) as i32,
        };

        Rect { pos, size }
    }
}

impl Drop for AtlasRegion {
    fn drop(&mut self) {
        // TODO: somehow this triggers sometimes
        // assert!(self.free);
    }
}

pub struct AtlasTextureCreator<'a> {
    pub atlas: &'a mut Atlas,
    pub sdl: &'a mut Sdl,
}

impl<'a> TileTextureProvider for AtlasTextureCreator<'a> {
    type Texture = AtlasRegion;

    fn alloc(&mut self, pixels_rgba: &[u8]) -> Self::Texture {
        let t = self.atlas.alloc(self.sdl);
        self.atlas.update(&t, pixels_rgba);
        t
    }

    fn free(&mut self, texture: Self::Texture) {
        self.atlas.remove(texture);
    }

    fn draw(&mut self, texture: &Self::Texture, to: Rect) {
        self.sdl.canvas_copy(
            &self.atlas.texture[texture.index.z as usize],
            Some(texture.rect_padded().to_sdl()),
            Some(to.to_sdl()),
        );
    }
}
