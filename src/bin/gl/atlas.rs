use crate::gl;
use crate::gl::types::GLuint;
use crate::gl::Gl;
use fractal_toy::atlas::AtlasRegion;
use fractal_toy::atlas::SimpleAtlas;
use fractal_toy::fractal::TileTextureProvider;

pub struct Atlas {
    pub simple: SimpleAtlas,
    pub texture: Vec<GLuint>,
}

impl Atlas {
    pub fn new() -> Self {
        Atlas {
            simple: SimpleAtlas::new(),
            texture: Vec::new(),
        }
    }

    fn create_page(&mut self, gl: &mut Gl) -> GLuint {
        let s = self.simple.size * self.simple.res;
        let mut texture = 0;
        unsafe {
            gl.GenTextures(1, &mut texture);
            gl.BindTexture(gl::TEXTURE_2D, texture);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as _);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as _);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
            gl.TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as _,
                s as _,
                s as _,
                0,
                gl::RGBA as _,
                gl::UNSIGNED_BYTE as _,
                std::ptr::null(),
            );
        }
        self.texture.push(texture);
        texture
    }

    pub fn alloc(&mut self, gl: &mut Gl, pixels: &[u8]) -> AtlasRegion {
        let region = self.simple.alloc();
        let texture = match self.texture.get_mut(region.index.z as usize) {
            Some(texture) => *texture,
            None => self.create_page(gl),
        };

        let rect = region.rect();
        unsafe {
            gl.BindTexture(gl::TEXTURE_2D, texture);
            gl.TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                rect.pos.x,
                rect.pos.y,
                rect.size.x,
                rect.size.y,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                pixels.as_ptr() as _,
            );
        }
        region
    }

    pub fn remove(&mut self, r: AtlasRegion) {
        self.simple.remove(r);
    }

    pub fn provider<'a>(&'a mut self, gl: &'a mut Gl) -> Provider<'a> {
        Provider { atlas: self, gl }
    }
}

impl Drop for Atlas {
    fn drop(&mut self) {
        unsafe {
            let gl = crate::static_gl();
            for texture in self.texture.drain(..) {
                gl.DeleteTextures(1, &texture);
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
