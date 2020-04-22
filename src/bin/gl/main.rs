use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::Window;
use glutin::window::WindowBuilder;
use glutin::{ContextBuilder, ContextWrapper, PossiblyCurrent};

use serial::atlas::*;
use serial::fractal::TileTextureProvider;
use serial::fractal::TEXTURE_SIZE;
use serial::math::*;
use serial::time::DeltaTime;
use serial::Fractal;
use serial::Input;

mod gl;
mod imm;
mod program;
mod shader;
use self::gl::Gl;
use self::imm::GfxImmState;
use self::imm::Vertex;

static mut GL: Option<Gl> = None;

unsafe fn static_gl() -> &'static mut Gl {
    match &mut GL {
        Some(x) => x,
        None => panic!(),
    }
}

struct GLCtx {
    pub size: Vector2<u32>,
    pub imm: GfxImmState,
    pub ctx: ContextWrapper<PossiblyCurrent, Window>,
}

impl GLCtx {
    fn new(window: WindowBuilder, event_loop: &EventLoop<()>) -> Self {
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

    fn gl(&mut self) -> &'static mut Gl {
        unsafe { static_gl() }
    }

    #[rustfmt::skip]
    fn draw(&mut self, atlas: &Atlas<gl::types::GLuint>, fractal: &Fractal<AtlasRegion>) {
        let gl = self.gl();

        unsafe {
            gl.ClearColor(1.0, 1.0, 1.0, 0.0);
            gl.Clear(gl::COLOR_BUFFER_BIT);
        }

        let mut texture = None;
        for (p, tile) in fractal.tiles.tiles.iter() {
            let r = fractal.pos.pos_to_rect(&p.pos);
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
                        texture = Some(tile.index.z);
                    }
                }
            }

            let tlx = tlx as f32 / (atlas.size*atlas.res) as f32;
            let tly = tly as f32 / (atlas.size*atlas.res) as f32;
            let thx = thx as f32 / (atlas.size*atlas.res) as f32;
            let thy = thy as f32 / (atlas.size*atlas.res) as f32;


            self.imm.push(Vertex { pos: [hx, hy], col: [1.0, 1.0, 1.0], tex: [ thx, thy ] });
            self.imm.push(Vertex { pos: [hx, ly], col: [1.0, 0.0, 1.0], tex: [ thx, tly ] });
            self.imm.push(Vertex { pos: [lx, ly], col: [0.0, 0.0, 1.0], tex: [ tlx, tly ] });

            self.imm.push(Vertex { pos: [lx, hy], col: [0.0, 1.0, 1.0], tex: [ tlx, thy ] });
            self.imm.push(Vertex { pos: [hx, hy], col: [1.0, 1.0, 1.0], tex: [ thx, thy ] });
            self.imm.push(Vertex { pos: [lx, ly], col: [0.0, 0.0, 1.0], tex: [ tlx, tly ] });
            self.imm.draw(atlas.texture[tile.index.z as usize] as i32, gl);
        }

        if let Some(texture) = texture {
            self.imm.draw(dbg!(atlas.texture[texture as usize]) as i32, gl);
        }

        self.ctx.swap_buffers().unwrap();
    }
}

impl Drop for GLCtx {
    fn drop(&mut self) {
        let gl = self.gl();
    }
}

fn handle_input(input: &mut Input, event: &WindowEvent) {
    use glutin::event;
    use glutin::event::ElementState::*;
    use glutin::event::MouseButton::*;
    use glutin::event::MouseScrollDelta::*;
    use glutin::event::VirtualKeyCode as VK;
    use glutin::event::WindowEvent::*;

    match event {
        CursorMoved { position, .. } => {
            input.mouse = Vector2::new(position.x as _, position.y as _)
        },
        MouseWheel { delta, .. } => match delta {
            LineDelta(_, y) => input.scroll = *y as i32,
            PixelDelta(_) => (),
        },
        MouseInput { button, state, .. } => match button {
            Left => input.mouse_down = *state == Pressed,
            _ => (),
        },
        KeyboardInput {
            input:
                event::KeyboardInput {
                    virtual_keycode: Some(key),
                    state,
                    ..
                },
            ..
        } => {
            let f = match *state {
                Pressed => 1.0,
                Released => 0.0,
            };

            match key {
                VK::W => input.dir_move.y = f,
                VK::S => input.dir_move.y = -f,
                VK::A => input.dir_move.x = f,
                VK::D => input.dir_move.x = -f,
                _ => {
                    dbg!(key);
                },
            }
        },
        _ => (),
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new();

    let mut ctx = GLCtx::new(window, &event_loop);
    let mut fractal: Fractal<AtlasRegion> = Fractal::new(Vector2::new(800, 600));
    let mut input = Input::new();
    let mut dt = DeltaTime(1.0 / 60.0);
    let mut atlas = Atlas::new();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(sz) => {
                    fractal.pos.resize(Vector2::new(sz.width, sz.height));
                    ctx.resize(sz.width, sz.height);
                },
                e => handle_input(&mut input, &e),
            },
            Event::MainEventsCleared => {
                fractal.do_input(&input, dt);
                input.begin();
                {
                    let mut p = imm::Provider { gl: ctx.gl() };
                    let mut p = AtlasTextureCreator {
                        atlas: &mut atlas,
                        sdl: &mut p,
                    };
                    fractal.update_tiles(&mut p);
                }
                ctx.draw(&atlas, &fractal);
            },
            Event::RedrawRequested(_) => {},
            _ => (),
        }
    });
}
