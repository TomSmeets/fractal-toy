use crate::sdl::Sdl;
use fractal_toy::atlas::AtlasRegion;
use fractal_toy::atlas::SimpleAtlas;
use fractal_toy::fractal::TileTextureProvider;
use fractal_toy::fractal::TEXTURE_SIZE;
use sdl2::render::Texture;
use crate::rect_to_sdl;

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

    pub fn alloc(&mut self, sdl: &mut Sdl, pixels: &[u8]) -> AtlasRegion {
        let region = self.simple.alloc();
        let texture = match self.texture.get_mut(region.index.z as usize) {
            Some(texture) => texture,
            None => {
                let s = self.simple.size * self.simple.res;
                let texture = sdl.create_texture_static_rgba8(s, s);
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

    pub fn provider<'a>(&'a mut self, sdl: &'a mut Sdl) -> Provider<'a> {
        Provider { atlas: self, sdl }
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

impl Default for Atlas {
    fn default() -> Self {
        Atlas::new()
    }
}

pub struct Provider<'a> {
    pub atlas: &'a mut Atlas,
    pub sdl: &'a mut Sdl,
}

impl TileTextureProvider for Provider<'_> {
    type Texture = AtlasRegion;

    fn alloc(&mut self, pixels_rgba: &[u8]) -> AtlasRegion {
        self.atlas.alloc(self.sdl, pixels_rgba)
    }

    fn free(&mut self, texture: AtlasRegion) {
        self.atlas.remove(texture)
    }
}
