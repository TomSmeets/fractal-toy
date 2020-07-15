use crate::gl;
use crate::gl::types::GLuint;
use crate::gl::Gl;
use fractal_toy::math::Rect;

pub struct TextureSettings {
    pub width: u32,
    pub height: u32,
}

pub struct Texture {
    id: GLuint,
}

impl Texture {
    pub fn new(gl: &mut Gl, param: &TextureSettings) -> Self {
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
                param.width as _,
                param.height as _,
                0,
                gl::RGBA as _,
                gl::UNSIGNED_BYTE as _,
                std::ptr::null(),
            );
        }

        Texture { id: texture }
    }

    pub fn id(&self) -> GLuint {
        self.id
    }

    pub fn write(&mut self, gl: &mut Gl, rect: Rect, pixels: &[u8]) {
        unsafe {
            gl.BindTexture(gl::TEXTURE_2D, self.id());
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
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            let gl = crate::static_gl();
            gl.DeleteTextures(1, &self.id);
        }
    }
}
