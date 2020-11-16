use crate::rect_to_sdl;
use crate::AtlasRegion;
use crate::SimpleAtlas;
use crate::TextureSizeAndPadding;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;

pub struct Atlas {
    pub simple: SimpleAtlas,
    pub texture: Vec<Texture>,
}

impl Atlas {
    pub fn new(tile_size: TextureSizeAndPadding) -> Self {
        Atlas {
            simple: SimpleAtlas::new(tile_size),
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
                let s = self.simple.size * self.simple.size_and_padding.size;
                let texture = texture_creator
                    .create_texture_static(PixelFormatEnum::ABGR8888, s, s)
                    .unwrap();
                self.texture.push(texture);
                self.texture.last_mut().unwrap()
            },
        };

        assert_eq!(pixels.len(), self.simple.size_and_padding.size_in_bytes());

        texture
            .update(
                Some(rect_to_sdl(region.rect(self.simple.size_and_padding))),
                pixels,
                self.simple.size_and_padding.stride_in_bytes(),
            )
            .unwrap();

        region
    }

    pub fn remove(&mut self, r: AtlasRegion) {
        self.simple.remove(r);
    }

    pub fn clear(&mut self) {
        self.simple = SimpleAtlas::new(self.simple.size_and_padding);
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
