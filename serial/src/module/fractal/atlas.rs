use crate::math::*;
use crate::module::Sdl;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Texture;

const PADDING: u32 = 2;

pub struct Atlas {
    pub free: Vec<Vector3<u32>>,
    pub size: u32,
    pub res: u32,

    pub texture: Vec<Texture>,
}

impl Atlas {
    pub fn new(res: u32) -> Atlas {
        let size = 64 * 64 / (res + PADDING);
        Atlas {
            free: Vec::new(),
            size,
            res,
            texture: Vec::new(),
        }
    }

    pub fn alloc_page(&mut self, sdl: &mut Sdl) {
        let page = self.texture.len();

        let texture = sdl
            .canvas
            .texture_creator()
            .create_texture_static(
                PixelFormatEnum::RGBA8888,
                self.size * (self.res + PADDING),
                self.size * (self.res + PADDING),
            )
            .unwrap();

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

        // draw a bit in the padding zone
        // Meh, this is not ideal, but it kind of wokrs
        {
            let mut r2 = r.rect();
            r2.pos.x += PADDING as i32 / 2;
            r2.pos.y += PADDING as i32 / 2;
            t.update(Some(r2.into_sdl()), pixels, 4 * self.res as usize)
                .unwrap();
        }
        {
            let mut r2 = r.rect();
            r2.pos.x -= PADDING as i32 / 2;
            r2.pos.y -= PADDING as i32 / 2;
            t.update(Some(r2.into_sdl()), pixels, 4 * self.res as usize)
                .unwrap();
        }

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

pub struct AtlasRegion {
    pub index: Vector3<u32>,
    res: u32,
    free: bool,
}

impl AtlasRegion {
    pub fn rect(&self) -> Rect {
        let pos = Vector2 {
            x: ((self.res + PADDING) * self.index.x + PADDING / 2) as i32,
            y: ((self.res + PADDING) * self.index.y + PADDING / 2) as i32,
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
        // TODO: somehow this triggers sometimes
        assert!(self.free);
    }
}
