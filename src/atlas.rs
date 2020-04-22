use crate::fractal::TileTextureProvider;
use crate::fractal::PADDING;
use crate::fractal::TEXTURE_SIZE;
use crate::math::*;
use serde::{Deserialize, Serialize};

pub trait AtlasTextureProvider {
    type Texture;

    fn alloc(&mut self, width: u32, height: u32) -> Self::Texture;
    fn update(&mut self, texture: &mut Self::Texture, rect: Rect, pixels: &[u8]);
    fn free(&mut self, texture: Self::Texture);
}

pub struct Atlas<T> {
    // number of tiles in both x and y
    pub size: u32,

    // resolution of one tile
    pub res: u32,

    // this is a free list of avalible spots where z is layer index in `texture`
    free: Vec<Vector3<u32>>,
    pub texture: Vec<T>,
}

impl<T> Atlas<T> {
    pub fn new() -> Self {
        let res = TEXTURE_SIZE as u32;
        // 4K is somewhere near the maximum texture size
        // determined by trail and error, size does not matter that much,
        let texture_size = 4 * 1024;
        let size = texture_size / res;
        Atlas {
            free: Vec::new(),
            size,
            res,
            texture: Vec::new(),
        }
    }

    pub fn alloc_page(&mut self, sdl: &mut impl AtlasTextureProvider<Texture = T>) {
        let page = self.texture.len();

        let texture = sdl.alloc(self.size * self.res, self.size * self.res);
        self.texture.push(texture);

        self.free.reserve(self.size as usize * self.size as usize);
        for j in 0..self.size {
            for i in 0..self.size {
                self.free.push(Vector3::new(i, j, page as u32));
            }
        }
    }

    pub fn alloc(
        &mut self,
        sdl: &mut impl AtlasTextureProvider<Texture = T>,
        pixels: &[u8],
    ) -> AtlasRegion {
        let r = match self.free.pop() {
            Some(i) => AtlasRegion {
                index: i,
                res: self.res,
                free: false,
            },
            None => {
                self.alloc_page(sdl);
                self.alloc(sdl, pixels)
            },
        };

        let r1 = r.rect();
        let t = &mut self.texture[r.index.z as usize];
        sdl.update(t, r1, pixels);
        r
    }

    pub fn remove(&mut self, mut r: AtlasRegion) {
        self.free.push(r.index);
        r.free = true;
    }
}

impl<T> Default for Atlas<T> {
    fn default() -> Self {
        Atlas::new()
    }
}

// impl Drop for Atlas<T> {
//     fn drop(&mut self) {
//         for t in self.texture.drain(..) {
//             unsafe {
//                 t.destroy();
//             }
//         }
//     }
// }

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
        // self.free should be true, however not when dropping the entire struct by serializing to disk
        // assert!(self.free);
    }
}

pub struct AtlasTextureCreator<'a, T: AtlasTextureProvider> {
    pub atlas: &'a mut Atlas<T::Texture>,
    pub sdl: &'a mut T,
}

impl<'a, T: AtlasTextureProvider> TileTextureProvider for AtlasTextureCreator<'a, T> {
    type Texture = AtlasRegion;

    fn alloc(&mut self, pixels_rgba: &[u8]) -> Self::Texture {
        self.atlas.alloc(self.sdl, pixels_rgba)
    }

    fn free(&mut self, texture: Self::Texture) {
        self.atlas.remove(texture)
    }
}
