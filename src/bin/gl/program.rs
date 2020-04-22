use super::shader::Shader;
use crate::gl;
use crate::gl::types::*;
use crate::gl::Gl;
use std::ffi::CStr;

#[derive(Debug)]
pub struct ProgramAttribs {
    pub pos: GLint,
    pub col: GLint,
}

#[derive(Debug)]
pub struct ShaderProgram {
    pub id: GLuint,
    pub frag: Shader,
    pub vert: Shader,
    pub attr: ProgramAttribs,
}

impl ShaderProgram {
    pub fn compile(gl: &Gl, vert_source: &CStr, frag_source: &CStr) -> Result<Self, String> {
        unsafe {
            let vert = Shader::compile(gl, gl::VERTEX_SHADER, vert_source)?;
            let frag = Shader::compile(gl, gl::FRAGMENT_SHADER, frag_source)?;
            let mut prog = ShaderProgram {
                id: gl.CreateProgram(),
                vert,
                frag,
                attr: ProgramAttribs { pos: -1, col: -1 },
            };
            gl.AttachShader(prog.id, prog.frag.id);
            gl.AttachShader(prog.id, prog.vert.id);
            gl.LinkProgram(prog.id);

            let mut success = 0;
            gl.GetProgramiv(prog.id, gl::LINK_STATUS, &mut success);
            if success == 0 {
                let log = prog.read_info_log(gl);
                return Err(log);
            }

            prog.attr.pos = gl.GetAttribLocation(
                prog.id,
                CStr::from_bytes_with_nul(b"vert_pos\0").unwrap().as_ptr(),
            );
            prog.attr.col = gl.GetAttribLocation(
                prog.id,
                CStr::from_bytes_with_nul(b"vert_col\0").unwrap().as_ptr(),
            );

            Ok(prog)
        }
    }

    pub fn use_program(&self, gl: &Gl) {
        unsafe {
            gl.UseProgram(self.id);
        }
    }

    fn read_info_log(&self, gl: &Gl) -> String {
        unsafe {
            let mut log_size = 0;
            gl.GetProgramiv(self.id, gl::INFO_LOG_LENGTH, &mut log_size);
            let mut log = vec![0_u8; log_size as _];
            gl.GetProgramInfoLog(self.id, log_size, &mut log_size, log.as_mut_ptr() as _);
            String::from_utf8(log).unwrap()
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            let gl = crate::static_gl();
            gl.DeleteProgram(self.id);
        }
        self.id = 0;
    }
}
