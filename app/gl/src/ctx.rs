use crate::atlas::Atlas;
use crate::gl::Gl;
use crate::imm::GfxImmState;
use crate::imm::Vertex;
use crate::static_gl;
use crate::GL;
use fractal_toy::math::*;
use fractal_toy::AtlasRegion;
use fractal_toy::Fractal;
use fractal_toy::UI;
use glutin::event_loop::EventLoop;
use glutin::window::Window;
use glutin::window::WindowBuilder;
use glutin::{ContextBuilder, ContextWrapper, PossiblyCurrent};

pub struct GLCtx {
    pub size: Vector2<u32>,
    pub imm: GfxImmState,
    pub ctx: ContextWrapper<PossiblyCurrent, Window>,
}

impl GLCtx {
    pub fn new(window: WindowBuilder, event_loop: &EventLoop<()>) -> Self {
        let ctx = ContextBuilder::new()
            .with_vsync(true)
            .build_windowed(window, event_loop)
            .unwrap();
        let ctx: ContextWrapper<PossiblyCurrent, _> = unsafe { ctx.make_current().unwrap() };

        let gl = Gl::load_with(|s| ctx.get_proc_address(s));

        unsafe {
            gl.Viewport(0, 0, 800, 600);
        }

        let ctx = GLCtx {
            size: Vector2::new(800, 600),
            ctx,
            imm: GfxImmState::new(&gl),
        };
        unsafe {
            GL = Some(gl);
        }
        ctx
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        self.size.x = w;
        self.size.y = h;

        unsafe {
            let gl = self.gl();
            gl.Viewport(0, 0, w as _, h as _);
        }
    }

    pub fn gl(&mut self) -> &'static mut Gl {
        unsafe { static_gl() }
    }

    #[rustfmt::skip]
    pub fn draw(&mut self, ui: &UI, atlas: &Atlas, fractal: &Fractal<AtlasRegion>) {
        let gl = self.gl();

        let mut texture = None;
        for (p, tile) in fractal.tiles.iter() {
            let r = fractal.pos.pos_to_rect(p);
            let lx = r.pos.x;
            let ly = r.pos.y;
            let hx = lx + r.size.x;
            let hy = ly + r.size.y;

            let size_x = self.size.x as f32;
            let size_y = self.size.y as f32;

            let lx = lx as f32 / size_x * 2.0 - 1.0;
            let ly = ly as f32 / size_y * 2.0 - 1.0;
            let hx = hx as f32 / size_x * 2.0 - 1.0;
            let hy = hy as f32 / size_y * 2.0 - 1.0;

            let ly = -ly;
            let hy = -hy;

            let r = tile.rect_padded();
            let tlx = r.pos.x;
            let tly = r.pos.y;
            let thx = r.pos.x + r.size.x;
            let thy = r.pos.y + r.size.y;

            match texture {
                None => texture = Some(tile.index.z),
                Some(t) => {
                    if t != tile.index.z {
                        self.imm.draw(atlas.texture[t as usize].id() as i32, gl);
                        texture = Some(tile.index.z);
                    }
                }
            }

            let pixel_size = atlas.simple.size * atlas.simple.res;
            let tlx = tlx as f32 / pixel_size as f32;
            let tly = tly as f32 / pixel_size as f32;
            let thx = thx as f32 / pixel_size as f32;
            let thy = thy as f32 / pixel_size as f32;


            // TODO: Thre is no need for color in here, remove it
            self.imm.push(Vertex { pos: [hx, hy], col: [1.0, 1.0, 1.0], tex: [ thx, thy ] });
            self.imm.push(Vertex { pos: [hx, ly], col: [1.0, 1.0, 1.0], tex: [ thx, tly ] });
            self.imm.push(Vertex { pos: [lx, ly], col: [1.0, 1.0, 1.0], tex: [ tlx, tly ] });

            self.imm.push(Vertex { pos: [lx, hy], col: [1.0, 1.0, 1.0], tex: [ tlx, thy ] });
            self.imm.push(Vertex { pos: [hx, hy], col: [1.0, 1.0, 1.0], tex: [ thx, thy ] });
            self.imm.push(Vertex { pos: [lx, ly], col: [1.0, 1.0, 1.0], tex: [ tlx, tly ] });
        }

        // what is this?
        if let Some(texture) = texture {
            self.imm.draw(atlas.texture[texture as usize].id() as i32, gl);
        }

        // TODO: this is very wrong and does not work, use a seperate shader for ui
        // ui should not be here?, in the future it should use its own texture atlas.
        if ! atlas.texture.is_empty() {
            for (rect, _) in ui.rects.iter() {
                let lx = (rect.pos.x ) as f32;
                let ly = (rect.pos.y ) as f32;
                let hx = (rect.pos.x + rect.size.x) as f32;
                let hy = (rect.pos.y + rect.size.y) as f32;

                let size_x = self.size.x as f32;
                let size_y = self.size.y as f32;

                let lx = lx as f32 / size_x * 2.0 - 1.0;
                let ly = ly as f32 / size_y * 2.0 - 1.0;
                let hx = hx as f32 / size_x * 2.0 - 1.0;
                let hy = hy as f32 / size_y * 2.0 - 1.0;

                let ly = - ly;
                let hy = - hy;

                let col = [ 1.0, 1.0, 1.0];

                self.imm.push(Vertex { pos: [hx, hy], col, tex: [ 1.0, 1.0 ] });
                self.imm.push(Vertex { pos: [hx, ly], col, tex: [ 1.0, 0.0 ] });
                self.imm.push(Vertex { pos: [lx, ly], col, tex: [ 0.0, 0.0 ] });
                self.imm.push(Vertex { pos: [lx, hy], col, tex: [ 0.0, 1.0 ] });
                self.imm.push(Vertex { pos: [hx, hy], col, tex: [ 1.0, 1.0 ] });
                self.imm.push(Vertex { pos: [lx, ly], col, tex: [ 0.0, 0.0 ] });
                self.imm.draw(atlas.texture[0].id() as i32, gl);
            }
        }
    }
}

impl Drop for GLCtx {
    fn drop(&mut self) {}
}
