use crate::math::*;
use crate::quadtree::pos::*;
use ::palette::*;

use sdl2::pixels::*;
use sdl2::rect::*;
use sdl2::render::*;

use crate::input::*;
use crate::quadtree::*;
use crate::sdl::*;
use crate::viewport::Viewport;
use crate::window::Window;
use crate::*;

pub enum DragState {
    None,
    From(V2),
}

pub struct Fractal {
    pub textures: QuadTree<Texture>,
    pub pos: Viewport,
    pub drag: DragState,
}

impl Fractal {
    pub fn new() -> Self {
        Fractal {
            textures: QuadTree::new(),
            pos: Viewport::new(),
            drag: DragState::None,
        }
    }

    pub fn update(&mut self, time: &Time, sdl: &mut Sdl, window: &Window, input: &Input) {
        let mouse_in_view = screen_to_view(window, input.mouse);
        self.pos.zoom_in(0.1 * input.scroll as f32, mouse_in_view);

        self.pos.translate(time.dt * input.dir_move);
        self.pos
            .zoom_in(time.dt * input.dir_look.y, V2::new(0.5, 0.5));

        if let DragState::From(p1) = self.drag {
            self.pos.translate(p1 - mouse_in_view);
        }

        self.drag = if input.mouse_down {
            DragState::From(mouse_in_view)
        } else {
            DragState::None
        };

        if input.is_down(InputAction::X) {
            self.textures.reduce_to(1);
        }
        if input.is_down(InputAction::Y) {
            self.textures.clear();
        }

        if input.is_down(InputAction::A) {
            let p = self.pos.get_pos();
            let t = mk_texture(&sdl.canvas.texture_creator(), p.clone());
            self.textures.insert_at(&p.path, t);
        }

        let vs = self.textures.values();
        for (p, v) in &vs {
            let r = self.pos_to_rect(window, p);
            sdl.canvas.copy(v, None, Some(r)).unwrap();
            sdl.canvas.set_draw_color(Color::RGB(255, 0, 0));
            sdl.canvas.draw_rect(r).unwrap();
        }

        {
            let w = 20;

            let mouse_view = screen_to_view(window, input.mouse);
            let mouse_world = self.pos.view_to_world(mouse_view);
            let mouse_view = self.pos.world_to_view(mouse_world);
            let mouse_screen = view_to_screen(window, mouse_view);

            sdl.canvas.set_draw_color(Color::RGB(255, 0, 0));
            sdl.canvas
                .fill_rect(Rect::from_center((mouse_screen.x, mouse_screen.y), w, w))
                .unwrap();

            // word space
            let p_min = V2::new(0., 0.);
            let p_max = V2::new(1., 1.);

            let p_min = self.pos.world_to_view(p_min);
            let p_min = view_to_screen(window, p_min);

            let p_max = self.pos.world_to_view(p_max);
            let p_max = view_to_screen(window, p_max);

            sdl.canvas.set_draw_color(Color::RGB(255, 0, 0));
            sdl.canvas.draw_rect(mk_rect(p_min, p_max)).unwrap();
        }

        {
            let r = self.pos_to_rect(window, &self.pos.get_pos());
            sdl.canvas.set_draw_color(Color::RGB(0, 255, 0));
            sdl.canvas.draw_rect(r).unwrap();
        }

        if input.is_down(InputAction::F1) {
            println!("---- INFO ----");
            self.info(input, window);
        }
    }

    fn pos_to_rect(&self, window: &Window, p: &QuadTreePosition) -> Rect {
        let (x, y, z) = p.float_top_left_with_size();
        let p = V2::new(x as f32, y as f32);
        let w = p + V2::new(z as f32, z as f32);
        let p = self.pos.world_to_view(p);
        let p = view_to_screen(window, p);
        let w = self.pos.world_to_view(w);
        let w = view_to_screen(window, w);
        mk_rect(p, w)
    }

    pub fn info(&self, input: &Input, window: &Window) {
        let mouse_view = screen_to_view(window, input.mouse);
        let mouse_world = self.pos.view_to_world(mouse_view);
        let mouse_view = self.pos.world_to_view(mouse_world);
        let mouse_screen = view_to_screen(window, mouse_view);
        println!("screen  {:6.2} {:6.2}", input.mouse.x, input.mouse.y);
        println!("view    {:6.2} {:6.2}", mouse_view.x, mouse_view.y);
        println!("world   {:6.2} {:6.2}", mouse_world.x, mouse_world.y);
        println!("screen2 {:6.2} {:6.2}", mouse_screen.x, mouse_screen.y);
    }
}

fn screen_to_view(window: &Window, p: V2i) -> V2 {
    V2::new(
        p.x as f32 / window.size.x as f32,
        1.0 - p.y as f32 / window.size.y as f32,
    )
}

fn view_to_screen(window: &Window, p: V2) -> V2i {
    V2i::new(
        (p.x * window.size.x as f32) as i32,
        ((1.0 - p.y) * window.size.y as f32) as i32,
    )
}

fn mk_texture<T>(canvas: &TextureCreator<T>, p: QuadTreePosition) -> Texture {
    let size = 256;
    let mut texture = canvas
        .create_texture_static(PixelFormatEnum::RGBA8888, size, size)
        .unwrap();
    let mut pixels = vec![0; (size * size * 4) as usize];
    draw_tile(&mut pixels, p);

    texture.update(None, &pixels, (4 * size) as usize).unwrap();
    texture
}

pub fn draw_tile(pixels: &mut [u8], p: QuadTreePosition) {
    let resolution: u32 = 256;
    // TODO: improve
    assert!(pixels.len() as u32 == resolution * resolution * 4);

    // gets center of this qpos square
    let (x, y, size) = p.float_top_left_with_size();
    let center = Vector2::new(x, y) * 4.0 - Vector2::new(2.0, 2.0);
    draw_mandel(pixels, resolution, resolution, size * 4.0, center);
}

fn mk_rect(a: V2i, b: V2i) -> Rect {
    let min_x = a.x.min(b.x);
    let min_y = a.y.min(b.y);

    let max_x = a.x.max(b.x);
    let max_y = a.y.max(b.y);

    let width = max_x - min_x;
    let height = max_y - min_y;

    Rect::new(min_x, min_y, width as u32, height as u32)
}

fn draw_mandel(pixels: &mut [u8], w: u32, h: u32, zoom: f64, offset: Vector2<f64>) {
    for y in 0..h {
        for x in 0..w {
            let mut c0 = Vector2::new(x as f64, y as f64);

            // screen coords 0 - 1
            c0.x /= w as f64;
            c0.y /= h as f64;
            c0.y = 1.0 - c0.y;

            // -1 , 1
            c0 = zoom * c0 + offset;

            let itr = mandel(256, c0);

            let mut v = itr as f32 / 256.0;
            v *= v;
            v = 1. - v;

            let hsv = Hsv::new(itr as f32 / 32.0 * 360., v, v);
            let rgb = Srgb::from(hsv).into_linear();

            pixels[(0 + (x + y * w) * 4) as usize] = 255;
            pixels[(1 + (x + y * w) * 4) as usize] = (rgb.red * 255.) as u8;
            pixels[(2 + (x + y * w) * 4) as usize] = (rgb.green * 255.) as u8;
            pixels[(3 + (x + y * w) * 4) as usize] = (rgb.blue * 255.) as u8;
        }
    }
}

fn mandel(max: i32, c: Vector2<f64>) -> i32 {
    let mut z = c;

    let mut n = 0;
    loop {
        let r = z.x;
        let i = z.y;
        z.x = r * r - i * i + c.x;
        z.y = 2.0 * r * i + c.y;

        if r * r + i * i > 4.0 {
            return n;
        }

        if n == max {
            return max;
        }
        n += 1;
    }
}
