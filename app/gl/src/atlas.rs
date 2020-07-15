use crate::gl::Gl;
use crate::texture::Texture;
use crate::texture::TextureSettings;
use fractal_toy::atlas::AtlasRegion;
use fractal_toy::atlas::SimpleAtlas;
use fractal_toy::fractal::TileTextureProvider;

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

    fn create_page(&mut self, gl: &mut Gl) -> &mut Texture {
        let s = self.simple.size * self.simple.res;

        let texture = Texture::new(gl, &TextureSettings {
            width: s,
            height: s,
        });

        self.texture.push(texture);
        self.texture.last_mut().unwrap()
    }

    pub fn alloc(&mut self, gl: &mut Gl, pixels: &[u8]) -> AtlasRegion {
        let region = self.simple.alloc();

        let texture = match self.texture.get_mut(region.index.z as usize) {
            Some(texture) => texture,
            None => self.create_page(gl),
        };

        let rect = region.rect();
        texture.write(gl, rect, pixels);
        region
    }

    pub fn remove(&mut self, r: AtlasRegion) {
        self.simple.remove(r);
    }

    pub fn provider<'a>(&'a mut self, gl: &'a mut Gl) -> Provider<'a> {
        Provider { atlas: self, gl }
    }
}

impl Default for Atlas {
    fn default() -> Self {
        Atlas::new()
    }
}

pub struct Provider<'a> {
    pub atlas: &'a mut Atlas,
    pub gl: &'a mut Gl,
}

impl TileTextureProvider for Provider<'_> {
    type Texture = AtlasRegion;

    fn alloc(&mut self, pixels_rgba: &[u8]) -> AtlasRegion {
        self.atlas.alloc(self.gl, pixels_rgba)
    }

    fn free(&mut self, texture: AtlasRegion) {
        self.atlas.remove(texture)
    }
}
