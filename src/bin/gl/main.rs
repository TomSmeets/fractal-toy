use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::Window;
use glutin::window::WindowBuilder;
use glutin::{ContextBuilder, ContextWrapper, PossiblyCurrent};

use serial::fractal::TileTextureProvider;
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

struct Provider {}
impl TileTextureProvider for Provider {
    type Texture = ();

    fn alloc(&mut self, _: &[u8]) {}

    fn free(&mut self, _: ()) {}
}

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

    fn draw(&mut self, fractal: &Fractal<()>) {
        let gl = self.gl();

        unsafe {
            gl.ClearColor(1.0, 1.0, 1.0, 0.0);
            gl.Clear(gl::COLOR_BUFFER_BIT);
        }

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

            self.imm.push(Vertex { pos: [lx, ly], col: [0.0, 0.0, 1.0]});
            self.imm.push(Vertex { pos: [hx, ly], col: [1.0, 0.0, 1.0]});
            self.imm.push(Vertex { pos: [hx, hy], col: [1.0, 1.0, 1.0]});

            self.imm.push(Vertex { pos: [lx, ly], col: [0.0, 0.0, 1.0]});
            self.imm.push(Vertex { pos: [hx, hy], col: [1.0, 1.0, 1.0]});
            self.imm.push(Vertex { pos: [lx, hy], col: [0.0, 1.0, 1.0]});
        }

        self.imm.draw(gl);
        self.ctx.swap_buffers().unwrap();
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new();

    let mut ctx = GLCtx::new(window, &event_loop);
    let mut fractal = Fractal::new(Vector2::new(800, 600));
    let mut input = Input::new();
    let mut dt = DeltaTime(1.0 / 60.0);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::CursorMoved { position, .. } => {
                    input.mouse = Vector2::new(position.x as _, position.y as _)
                },
                WindowEvent::Resized(sz) => {
                    fractal.pos.resize(Vector2::new(sz.width, sz.height));
                    ctx.resize(sz.width, sz.height);
                },
                _ => (),
            },
            Event::MainEventsCleared => {
                let mut p = Provider {};
                fractal.do_input(&input, dt);
                input.begin();

                fractal.update_tiles(&mut p);
                ctx.draw(&fractal);
            },
            Event::RedrawRequested(_) => {},
            _ => (),
        }
    });
}
