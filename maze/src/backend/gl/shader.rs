use super::raw as gl;
use super::raw::types::*;
use super::raw::Gl;
use std::ffi::*;

#[derive(Debug)]
pub struct Shader {
    pub id: GLuint,
}

impl Shader {
    pub fn compile(gl: &Gl, shader_type: GLuint, source: &CStr) -> Result<Self, String> {
        unsafe {
            let shader = match gl.CreateShader(shader_type) {
                0 => return Err(String::from("failed to create shader")),
                id => Shader { id },
            };

            let sources = [source.as_ptr()];
            gl.ShaderSource(shader.id, 1, sources.as_ptr(), 0 as _);
            gl.CompileShader(shader.id);

            let mut success = 0;
            gl.GetShaderiv(shader.id, gl::COMPILE_STATUS, &mut success);
            if success == 0 {
                let shader_log = shader.read_info_log(gl);
                return Err(shader_log);
            }
            Ok(shader)
        }
    }

    fn read_info_log(&self, gl: &Gl) -> String {
        unsafe {
            let mut log_size = 0;
            gl.GetShaderiv(self.id, gl::INFO_LOG_LENGTH, &mut log_size);
            let mut log = vec![0_u8; log_size as _];
            gl.GetShaderInfoLog(self.id, log_size, &mut log_size, log.as_mut_ptr() as _);
            String::from_utf8(log).unwrap()
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            if let Some(gl) = super::GL.as_ref() {
                gl.DeleteShader(self.id);
            }
        }
        self.id = 0;
    }
}
