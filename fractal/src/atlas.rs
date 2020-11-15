use crate::math::*;
use crate::TextureSizeAndPadding;
use serde::{Deserialize, Serialize};

pub struct SimpleAtlas {
    pub size_and_padding: TextureSizeAndPadding,

    // number of tiles in both x and y
    pub size: u32,
    pub page_count: u32,

    // free tiles
    free: Vec<Vector3<u32>>,
}

impl SimpleAtlas {
    pub fn new(size_and_padding: TextureSizeAndPadding) -> Self {
        let res = size_and_padding.size;

        // 4K is somewhere near the maximum texture size
        // determined by trail and error, size does not matter that much,
        let texture_size = 4 * 1024;
        let size = texture_size / res;

        SimpleAtlas {
            size_and_padding,
            free: Vec::new(),
            size,
            page_count: 0,
        }
    }

    pub fn alloc_page(&mut self) {
        let page = self.page_count;
        self.page_count += 1;
        self.free.reserve(self.size as usize * self.size as usize);
        for j in 0..self.size {
            for i in 0..self.size {
                self.free.push(Vector3::new(i, j, page as u32));
            }
        }
    }

    pub fn alloc(&mut self) -> AtlasRegion {
        match self.free.pop() {
            Some(i) => AtlasRegion {
                index: i,
                free: false,
            },
            None => {
                self.alloc_page();
                self.alloc()
            },
        }
    }

    pub fn remove(&mut self, mut r: AtlasRegion) {
        self.free.push(r.index);
        r.free = true;
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AtlasRegion {
    pub index: Vector3<u32>,
    free: bool,
}

impl AtlasRegion {
    pub fn rect(&self, texture: TextureSizeAndPadding) -> Rect {
        let pos = Vector2 {
            x: (texture.size * self.index.x) as i32,
            y: (texture.size * self.index.y) as i32,
        };

        let size = Vector2 {
            x: (texture.size) as i32,
            y: (texture.size) as i32,
        };

        Rect { pos, size }
    }

    // returns smaller rectangle with padding removed
    pub fn rect_padded(&self, texture: TextureSizeAndPadding) -> Rect {
        let pos = Vector2 {
            x: (texture.size * self.index.x) as i32 + texture.padding as i32,
            y: (texture.size * self.index.y) as i32 + texture.padding as i32,
        };

        let size = Vector2 {
            x: (texture.size as i32 - texture.padding as i32 * 2) as i32,
            y: (texture.size as i32 - texture.padding as i32 * 2) as i32,
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
