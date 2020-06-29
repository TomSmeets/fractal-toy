use super::program::ShaderProgram;
use crate::gl;
use crate::gl::types::*;
use crate::gl::Gl;
use memoffset::offset_of;
use std::ffi::CString;

#[repr(C)]
#[derive(Debug)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub col: [f32; 3],
    pub tex: [f32; 2],
}

#[derive(Debug)]
pub struct GfxImmState {
    gl_vao: GLuint,
    gl_verts: GLuint,
    gl_texture: GLuint,

    shader: ShaderProgram,

    vertex: Vec<Vertex>,
}

impl GfxImmState {
    pub fn new(gl: &Gl) -> Self {
        let mut gl_vao = 0;
        let mut gl_verts = 0;
        unsafe {
            gl.GenVertexArrays(1, &mut gl_vao);
            gl.GenBuffers(1, &mut gl_verts);
        }

        let vert = include_str!("main.vert");
        let frag = include_str!("main.frag");
        let vert = CString::new(vert).unwrap();
        let frag = CString::new(frag).unwrap();

        let shader = ShaderProgram::compile(gl, &vert, &frag).unwrap();

        let gl_texture = 0;
        unsafe {
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as _);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as _);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
        }

        GfxImmState {
            gl_vao,
            gl_verts,
            gl_texture,
            shader,
            vertex: Vec::new(),
        }
    }

    pub fn push(&mut self, v: Vertex) {
        self.vertex.push(v);
    }

    pub fn draw(&mut self, texture: gl::types::GLint, gl: &mut Gl) {
        self.shader.use_program(gl);
        unsafe {
            gl.BindVertexArray(self.gl_vao);

            gl.BindBuffer(gl::ARRAY_BUFFER, self.gl_verts);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (self.vertex.len() * std::mem::size_of::<Vertex>()) as isize,
                self.vertex.as_ptr() as _,
                gl::DYNAMIC_DRAW,
            );

            gl.VertexAttribPointer(
                self.shader.attr.pos as _,
                2,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<Vertex>() as _,
                offset_of!(Vertex, pos) as _,
            );
            gl.EnableVertexAttribArray(self.shader.attr.pos as _);
            gl.VertexAttribPointer(
                self.shader.attr.col as _,
                3,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<Vertex>() as _,
                offset_of!(Vertex, col) as _,
            );
            gl.EnableVertexAttribArray(self.shader.attr.col as _);
            gl.VertexAttribPointer(
                self.shader.attr.tex as _,
                2,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<Vertex>() as _,
                offset_of!(Vertex, tex) as _,
            );
            gl.EnableVertexAttribArray(self.shader.attr.tex as _);

            if true {
                gl.Enable(gl::CULL_FACE);
                gl.CullFace(gl::BACK);
            }

            gl.BindTexture(gl::TEXTURE_2D, texture as _);
            gl.DrawArrays(gl::TRIANGLES, 0, self.vertex.len() as i32);
        }

        self.vertex.clear();
    }
}

impl Drop for GfxImmState {
    fn drop(&mut self) {
        unsafe {
            let gl = crate::static_gl();
            gl.DeleteVertexArrays(1, &self.gl_vao);
            gl.DeleteBuffers(1, &self.gl_verts);
        }
    }
}
