use crate::math::*;
use crate::quadtree::pos::*;
use ::palette::*;

use sdl2::pixels::*;
use sdl2::rect::*;
use sdl2::render::*;

use std::collections::hash_map::{Entry, HashMap};

use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::input::{Input, InputAction};
use crate::sdl::Sdl;
use crate::viewport::Viewport;
use crate::window::Window;
use crate::*;

pub mod tile;
use self::tile::*;

static TEXTURE_SIZE: usize = 64;

pub enum DragState {
    None,
    From(V2),
}

pub enum TileState {
    Queued,
    Working,
    Done(TileContent),
}

pub struct TileContent {
    pixels: Vec<u8>,
}

impl TileContent {
    pub fn new(p: TilePos) -> TileContent {
        let mut pixels = vec![0; (TEXTURE_SIZE * TEXTURE_SIZE * 4) as usize];
        draw_tile(pixels.as_mut_slice(), p);
        TileContent { pixels }
    }

    pub fn to_sdl(&self, texture: &mut Texture) {
        texture
            .update(None, &self.pixels, (4 * TEXTURE_SIZE) as usize)
            .unwrap();
    }
}

pub struct Fractal {
    pub textures: Arc<Mutex<HashMap<TilePos, TileState>>>,
    pub pos: Viewport,
    pub drag: DragState,
}

impl Fractal {
    pub fn new() -> Self {
        let h: HashMap<TilePos, TileState> = HashMap::new();
        let q = Arc::new(Mutex::new(h));

        for i in 0..6 {
            let q = q.clone();
            thread::spawn(move || loop {
                let next: Option<TilePos> = {
                    let mut l = q.lock().unwrap();
                    let p = l
                        .iter()
                        .filter(|(_, x)| matches!(x, TileState::Queued))
                        .map(|(p, _)| *p)
                        .max_by_key(|p| p.z);

                    if let Some(p) = p {
                        l.insert(p, TileState::Working);
                    }
                    p
                };

                match next {
                    Some(p) => {
                        let t = TileContent::new(p);
                        let mut map = q.lock().unwrap();
                        let old = map.insert(p, TileState::Done(t));
                        if let Some(TileState::Done(_)) = old {
                            println!("Duplicate work!");
                        }
                    },
                    None => {
                        thread::yield_now();
                    },
                }
            });
        }

        Fractal {
            textures: q,
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

        self.drag = if input.mouse_down.is_down {
            DragState::From(mouse_in_view)
        } else {
            DragState::None
        };

        if input.is_down(InputAction::Y) {
            let t = self.textures.lock();
            t.unwrap().clear();
        }

        // if input.is_down(InputAction::A) {
        {
            let ps = self.pos.get_pos_all();
            let mut t = self.textures.lock().unwrap();
            for p in ps {
                let e = t.entry(p);
                match e {
                    Entry::Occupied(e) => {},
                    Entry::Vacant(e) => {
                        e.insert(TileState::Queued);
                    },
                }
            }
        }

        {
            let t = self.textures.lock().unwrap();
            let mut vs: Vec<_> = t.iter().collect();
            vs.sort_unstable_by_key(|(p, _)| p.z);
            let mut texture = sdl
                .canvas
                .texture_creator()
                .create_texture_static(
                    PixelFormatEnum::RGBA8888,
                    TEXTURE_SIZE as u32,
                    TEXTURE_SIZE as u32,
                )
                .unwrap();

            let mut count_empty = 0;
            let mut count_full = 0;
            for (p, v) in &vs {
                if let TileState::Done(v) = v {
                    v.to_sdl(&mut texture);
                    let r = self.pos_to_rect(window, p);
                    sdl.canvas.copy(&texture, None, Some(r)).unwrap();
                    count_full += 1;
                } else {
                    count_empty += 1;
                }
            }

            println!("{} / {}", count_empty, count_empty + count_full);

            unsafe {
                texture.destroy();
            }
        }

        if true {
            let self_zoom = self.pos.zoom;
            self.textures.lock().unwrap().retain(|p, t| {
                let z_min = self_zoom - 1.0;
                let z_max = self_zoom + 5.0;
                let keep = (p.z as f32) > z_min && (p.z as f32) < z_max;
                keep
            });
        }

        if input.is_down(InputAction::F1) {
            println!("---- INFO ----");
            self.info(input, window);
        }
    }

    fn pos_to_rect(&self, window: &Window, p: &TilePos) -> Rect {
        let [x, y, z, _] = p.to_f32();
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

fn mk_texture<T>(canvas: &TextureCreator<T>, p: TilePos) -> Texture {
    let size = 256;
    let mut texture = canvas
        .create_texture_static(PixelFormatEnum::RGBA8888, size, size)
        .unwrap();
    let mut pixels = vec![0; (size * size * 4) as usize];
    draw_tile(&mut pixels, p);

    texture.update(None, &pixels, (4 * size) as usize).unwrap();
    texture
}

pub fn draw_tile(pixels: &mut [u8], p: TilePos) {
    let [x, y, size] = p.to_f64();
    let center = Vector2::new(x, y) * 4.0 - Vector2::new(2.0, 2.0);
    draw_mandel(
        pixels,
        TEXTURE_SIZE as u32,
        TEXTURE_SIZE as u32,
        size * 4.0,
        center,
    );
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

// TODO: profile!!
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
