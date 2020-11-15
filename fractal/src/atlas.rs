use crate::fractal::PADDING;
use crate::fractal::TEXTURE_SIZE;
use crate::math::*;
use serde::{Deserialize, Serialize};

pub struct SimpleAtlas {
    // number of tiles in both x and y
    pub size: u32,

    // resolution of one tile
    pub res: u32,

    pub page_count: u32,

    // free tiles
    free: Vec<Vector3<u32>>,
}

impl SimpleAtlas {
    pub fn new() -> Self {
        let res = TEXTURE_SIZE as u32;
        // 4K is somewhere near the maximum texture size
        // determined by trail and error, size does not matter that much,
        let texture_size = 4 * 1024;
        let size = texture_size / res;
        SimpleAtlas {
            free: Vec::new(),
            size,
            res,
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
                res: self.res,
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

impl Default for SimpleAtlas {
    fn default() -> Self {
        SimpleAtlas::new()
    }
}


#[derive(Clone, Serialize, Deserialize)]
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
