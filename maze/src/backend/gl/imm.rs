use super::program::ShaderProgram;
use super::raw as gl;
use super::raw::types::*;
use super::raw::Gl;
use memoffset::offset_of;
use std::ffi::CString;

#[repr(C)]
#[derive(Debug)]
pub struct Vertex {
    pos: [f32; 2],
    col: [f32; 3],
}

#[derive(Debug)]
pub struct GfxImmState {
    gl_vao: GLuint,
    gl_verts: GLuint,
    gl_elems: GLuint,

    shader: ShaderProgram,

    vertex: Vec<Vertex>,
    index: Vec<u32>,
}

impl GfxImmState {
    pub fn new(gl: &Gl) -> Self {
        let mut gl_vao = 0;
        let mut gl_verts = 0;
        let mut gl_elems = 0;
        unsafe {
            gl.GenVertexArrays(1, &mut gl_vao);
            gl.GenBuffers(1, &mut gl_verts);
            gl.GenBuffers(1, &mut gl_elems);
        }

        let vert = include_str!("main.vert");
        let frag = include_str!("main.frag");
        let vert = CString::new(vert).unwrap();
        let frag = CString::new(frag).unwrap();

        let shader = ShaderProgram::compile(gl, &vert, &frag).unwrap();

        GfxImmState {
            gl_vao,
            gl_verts,
            gl_elems,
            shader,
            vertex: Vec::new(),
            index: Vec::new(),
        }
    }

    pub fn push(&mut self, v: Vertex) -> u32 {
        let i = self.vertex.len();
        self.vertex.push(v);
        i as u32
    }

    pub fn push_tri(&mut self, i: u32, j: u32, k: u32) {
        self.index.push(i);
        self.index.push(j);
        self.index.push(k);
    }

    pub fn rect(&mut self, rect: [f32; 4], col: [f32; 3]) {
        let lx = rect[0];
        let ly = rect[1];
        let hx = lx + rect[2];
        let hy = ly + rect[3];

        let ll = self.push(Vertex { pos: [lx, ly], col });
        let hh = self.push(Vertex { pos: [hx, hy], col });
        let lh = self.push(Vertex { pos: [lx, hy], col });
        let hl = self.push(Vertex { pos: [hx, ly], col });

        self.push_tri(ll, hh, lh);
        self.push_tri(ll, hl, hh);
    }

    pub fn draw(&mut self, gl: &Gl) {
        self.shader.use_program(gl);
        unsafe {
            if true {
                gl.Enable(gl::CULL_FACE);
                gl.CullFace(gl::BACK);
            }

            gl.BindVertexArray(self.gl_vao);

            gl.BindBuffer(gl::ARRAY_BUFFER, self.gl_verts);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (self.vertex.len() * std::mem::size_of::<Vertex>()) as isize,
                self.vertex.as_ptr() as _,
                gl::DYNAMIC_DRAW,
            );

            gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.gl_elems);
            gl.BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (self.index.len() * std::mem::size_of::<u32>()) as isize,
                self.index.as_ptr() as _,
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

            // glBindTexture(GL_TEXTURE_2D, g_gl.texture);
            // gl.DrawArrays(gl::TRIANGLES, 0, self.vertex.len() as i32);
            gl.DrawElements(
                gl::TRIANGLES,
                self.index.len() as i32,
                gl::UNSIGNED_INT,
                0 as _,
            );
        }

        self.vertex.clear();
        self.index.clear();
    }
}

impl Drop for GfxImmState {
    fn drop(&mut self) {
        unsafe {
            let gl = super::GL.as_ref().unwrap();
            gl.DeleteVertexArrays(1, &self.gl_vao);
            gl.DeleteBuffers(1, &self.gl_verts);
            gl.DeleteBuffers(1, &self.gl_elems);
        }
    }
}
