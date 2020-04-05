use crate::{math::*, module::Sdl};
use sdl2::{pixels::PixelFormatEnum, render::Texture};
use serde::{Deserialize, Serialize};

pub const PADDING: u32 = 1;

#[derive(Serialize, Deserialize)]
pub struct Atlas {
    pub size: u32,
    pub res: u32,

    #[serde(skip)]
    pub free: Vec<Vector3<u32>>,

    #[serde(skip)]
    pub texture: Vec<Texture>,
}

impl Atlas {
    pub fn new(res: u32) -> Atlas {
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
        t.update(Some(r1.into_sdl()), pixels, 4 * self.res as usize)
            .unwrap();
    }

    pub fn remove(&mut self, mut r: AtlasRegion) {
        self.free.push(r.index);
        r.free = true;
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
