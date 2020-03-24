use glutin::event::Event;
use glutin::event::VirtualKeyCode;
use glutin::event::WindowEvent;
use std::time::Instant;

use crate::gl;
use crate::gl::types::*;
use crate::gl::Gl;
use crate::Platform;

use std::ffi::*;

use memoffset::offset_of;

#[repr(C)]
#[derive(Debug)]
struct Vertex {
    pos: [f32; 2],
    col: [f32; 3],
}

#[derive(Debug)]
pub struct GfxImmState {
    gl_vao: GLuint,
    gl_verts: GLuint,
    gl_elements: GLuint,

    shader: ShaderProgram,

    vertex: Vec<Vertex>,
    index: Vec<u16>,
}

#[derive(Debug)]
pub struct Shader {
    id: GLuint,
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
        let gl = &crate::platform().gl;
        unsafe {
            gl.DeleteShader(self.id);
        }
        self.id = 0;
    }
}

#[derive(Debug)]
pub struct ShaderProgram {
    id: GLuint,
    frag: Shader,
    vert: Shader,
}

impl ShaderProgram {
    pub fn compile(gl: &Gl, vert_source: &CStr, frag_source: &CStr) -> Result<Self, String> {
        unsafe {
            let vert = Shader::compile(gl, gl::VERTEX_SHADER, vert_source)?;
            let frag = Shader::compile(gl, gl::FRAGMENT_SHADER, frag_source)?;
            let prog = ShaderProgram {
                id: gl.CreateProgram(),
                vert,
                frag,
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

impl GfxImmState {
    pub fn new(gl: &Gl) -> Self {
        let mut gl_vao = 0;
        let mut gl_verts = 0;
        let mut gl_elements = 0;
        unsafe {
            gl.GenVertexArrays(1, &mut gl_vao);
            gl.GenBuffers(1, &mut gl_verts);
            gl.GenBuffers(1, &mut gl_elements);
        }

        let vert = include_str!("main.vert");
        let frag = include_str!("main.frag");
        let vert = CString::new(vert).unwrap();
        let frag = CString::new(frag).unwrap();

        let shader = ShaderProgram::compile(gl, &vert, &frag).unwrap();

        GfxImmState {
            gl_vao,
            gl_verts,
            gl_elements,
            shader,
            vertex: Vec::new(),
            index: Vec::new(),
        }
    }

    /*
    static u32 gl_shader_create(GLShader *shader) {
      shader->vert = vert;
      shader->frag = frag;

      GLuint p = glCreateProgram();

      if(!p) {
        printf("Failed to create shader program!\n");
        return 0;
      }

      glAttachShader(p, frag);
      glAttachShader(p, vert);
      glLinkProgram(p);

      i32 success;
      glGetProgramiv(p, GL_LINK_STATUS, &success);
      if(!success) {
        puts("Failed to link shader!");
        GLint logSize = 0;
        glGetProgramiv(p, GL_INFO_LOG_LENGTH, &logSize);
        char shader_log[logSize];
        glGetProgramInfoLog(p, logSize, NULL, shader_log);
        fputs(shader_log, stderr);
        glDeleteProgram(p);
        return 0;
      }

      shader->prog = p;
      glUseProgram(p);
      return 1;
    }

    */

    pub fn draw(&mut self) {
        let gl = &mut crate::platform().gl;
        self.vertex.push(Vertex {
            pos: [0.0, 1.0],
            col: [1.0, 0.0, 0.0],
        });
        self.vertex.push(Vertex {
            pos: [-1.0, -1.0],
            col: [0.0, 1.0, 0.0],
        });
        self.vertex.push(Vertex {
            pos: [1.0, -1.0],
            col: [0.0, 0.0, 1.0],
        });

        self.index.push(0);
        self.index.push(1);
        self.index.push(2);
        self.index.push(2);
        self.index.push(1);
        self.index.push(0);
        self.shader.use_program(gl);
        unsafe {
            // gl.Viewport(0, 0, 200, 200);

            gl.BindVertexArray(self.gl_vao);

            gl.BindBuffer(gl::ARRAY_BUFFER, self.gl_verts);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (self.vertex.len() * std::mem::size_of::<Vertex>()) as isize,
                self.vertex.as_ptr() as _,
                gl::DYNAMIC_DRAW,
            );

            // gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.gl_elements);
            // gl.BufferData(
            //     gl::ELEMENT_ARRAY_BUFFER,
            //     (self.index.len() * std::mem::size_of::<u16>()) as isize,
            //     self.index.as_ptr() as _,
            //     gl::DYNAMIC_DRAW,
            // );

            let attrib_pos = 0;
            let attrib_col = 1;
            gl.VertexAttribPointer(
                attrib_pos,
                2,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<Vertex>() as _,
                offset_of!(Vertex, pos) as _,
            );
            gl.EnableVertexAttribArray(attrib_pos);
            gl.VertexAttribPointer(
                attrib_col,
                3,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<Vertex>() as _,
                offset_of!(Vertex, col) as _,
            );
            gl.EnableVertexAttribArray(attrib_col);

            /*
            GLShader &shader = *gl_current_shader();

            if(shader.attribute.in_pos >= 0) {
              glEnableVertexAttribArray(shader.attribute.in_pos);
              glVertexAttribPointer(shader.attribute.in_pos, 3, GL_FLOAT, GL_FALSE, sizeof(Vertex), &v0->pos);
            }

            if(shader.attribute.in_col >= 0) {
              glEnableVertexAttribArray(shader.attribute.in_col);
              glVertexAttribPointer(shader.attribute.in_col, 4, GL_FLOAT, GL_FALSE, sizeof(Vertex), &v0->color);
            }

            if(shader.attribute.in_tex >= 0) {
              glEnableVertexAttribArray(shader.attribute.in_tex);
              glVertexAttribPointer(shader.attribute.in_tex, 2, GL_FLOAT, GL_FALSE, sizeof(Vertex), &v0->uv);
            }
            */
            // glBindTexture(GL_TEXTURE_2D, g_gl.texture);
            gl.DrawArrays(gl::TRIANGLES, 0, self.index.len() as i32);
        }

        self.index.clear();
        self.vertex.clear();
    }
}

impl Drop for GfxImmState {
    fn drop(&mut self) {
        let gl = &mut crate::platform().gl;
        unsafe {
            gl.DeleteVertexArrays(1, &self.gl_vao);
            gl.DeleteBuffers(1, &self.gl_verts);
            gl.DeleteBuffers(1, &self.gl_elements);
        }
    }
}
