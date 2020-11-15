use crate::rect_to_sdl;
use fractal_toy::AtlasRegion;
use fractal_toy::SimpleAtlas;
use fractal_toy::TEXTURE_SIZE;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;

pub struct Atlas {
    pub simple: SimpleAtlas,
    pub texture: Vec<Texture>,
}

impl Atlas {
    pub fn new() -> Self {
        Atlas {
            simple: SimpleAtlas::new(),
            texture: Vec::new(),
        }
    }

    pub fn alloc(
        &mut self,
        texture_creator: &TextureCreator<WindowContext>,
        pixels: &[u8],
    ) -> AtlasRegion {
        let region = self.simple.alloc();
        let texture = match self.texture.get_mut(region.index.z as usize) {
            Some(texture) => texture,
            None => {
                let s = self.simple.size * self.simple.res;
                let texture = texture_creator
                    .create_texture_static(PixelFormatEnum::ABGR8888, s, s)
                    .unwrap();
                self.texture.push(texture);
                self.texture.last_mut().unwrap()
            },
        };

        texture
            .update(
                Some(rect_to_sdl(region.rect())),
                pixels,
                4 * TEXTURE_SIZE as usize,
            )
            .unwrap();

        region
    }

    pub fn remove(&mut self, r: AtlasRegion) {
        self.simple.remove(r);
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
