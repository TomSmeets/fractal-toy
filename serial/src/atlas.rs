use crate::math::*;
use crate::sdl::Sdl;
use crate::ui::Rect;
use sdl2::pixels::*;
use sdl2::rect::*;
use sdl2::render::Texture;
use sdl2::render::*;

pub struct Atlas {
    pub free: Vec<Vector2<u32>>,
    pub size: Vector2<u32>,
    pub res: u32,

    pub texture: Texture,
}

impl Atlas {
    pub fn new(sdl: &mut Sdl, size: Vector2<u32>, res: u32) -> Atlas {
        let mut free = Vec::with_capacity(size.x as usize * size.y as usize);
        for j in 0..size.y {
            for i in 0..size.x {
                free.push(Vector2::new(i, j));
            }
        }

        let texture = sdl
            .canvas
            .texture_creator()
            .create_texture_streaming(PixelFormatEnum::RGBA8888, size.x * res, size.y * res)
            .unwrap();

        Atlas {
            free,
            size,
            res,
            texture,
        }
    }

    pub fn alloc(&mut self) -> Option<AtlasRegion> {
        let i = self.free.pop()?;
        Some(AtlasRegion {
            index: i,
            res: self.res,
            free: false,
        })
    }

    pub fn update(&mut self, r: &AtlasRegion, pixels: &[u8]) {
        self.texture
            .update(Some(r.rect().into_sdl()), pixels, 4 * self.res as usize)
            .unwrap();
    }

    pub fn remove(&mut self, mut r: AtlasRegion) {
        self.free.push(r.index);
        r.free = true;
    }
}

pub struct AtlasRegion {
    index: Vector2<u32>,
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
            x: self.res as i32,
            y: self.res as i32,
        };
        Rect { pos, size }
    }
}

impl Drop for AtlasRegion {
    fn drop(&mut self) {
        assert!(self.free);
    }
}
