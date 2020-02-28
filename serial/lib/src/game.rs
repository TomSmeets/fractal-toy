use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::mouse::*;
use sdl2::pixels::*;
use sdl2::rect::*;
use sdl2::render::*;

use crate::fractal::*;
use crate::input::*;
use crate::math::*;
use crate::quadtree::pos::*;
use crate::quadtree::*;
use crate::sdl::*;

// TODO: implemnt save and load, this will handle some types that dont work with reload.
// For example the btreemap
pub struct State {
    sdl: Sdl,
    input: Input,
    textures: QuadTree<Texture>,

    pos: QuadTreePosition,

    offset: Vector2<f64>,
    zoom: f64,

    window_size: Vector2<u32>,
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

impl Default for State {
    fn default() -> State {
        State::new()
    }
}

impl State {
    pub fn unload(&mut self) {}
    pub fn reload(&mut self) {}

    pub fn new() -> State {
        let sdl = Sdl::new();

        // TODO: get window size
        State {
            sdl,
            pos: QuadTreePosition::root(),
            input: Input::new(),
            textures: QuadTree::new(),
            offset: Vector2::zero(),
            zoom: 1.0,
            window_size: Vector2::new(800, 600),
        }
    }

    pub fn update(&mut self) -> bool {
        let mut quit = false;

        let dt = 1.0 / 60.0;

        let mut down = false;
        for event in self.sdl.event.poll_iter() {
            self.input.handle_sdl(&event);
            match event {
                Event::Quit { .. } => {
                    quit = true;
                }
                Event::KeyDown {
                    keycode: Some(key), ..
                } => match key {
                    Keycode::Q => quit = true,
                    Keycode::C => down = true,
                    Keycode::R => {
                        self.textures.reduce_to(1);
                    }
                    Keycode::F => self.textures.clear(),
                    Keycode::Num0 => self.pos.parent(),
                    Keycode::Num1 => self.pos.child(0, 0),
                    Keycode::Num2 => self.pos.child(1, 0),
                    Keycode::Num3 => self.pos.child(0, 1),
                    Keycode::Num4 => self.pos.child(1, 1),
                    _ => (),
                },

                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Right,
                    ..
                } => {
                    self.pos.parent();
                }

                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Left,
                    x,
                    y,
                    ..
                } => {
                    let qx = (x * 2) / self.window_size.x as i32;
                    let qy = (y * 2) / self.window_size.x as i32;
                    self.pos.child(qx as u8, qy as u8);
                    let t = mk_texture(&self.sdl.canvas.texture_creator(), self.pos.clone());
                    self.textures.at(&self.pos.path).unwrap().value = Some(t);
                }

                Event::MouseWheel { y, .. } => {
                    self.zoom += 0.5 * (y as f64);
                }

                Event::Window {
                    win_event: WindowEvent::Resized(x, y),
                    ..
                } => {
                    self.window_size.x = (x as u32).max(1);
                    self.window_size.y = (y as u32).max(1);
                }

                _ => {}
            }
        }

        self.offset += dt * self.input.dir_move * 0.5_f64.powf(self.zoom);
        self.zoom += 2.0 * dt * self.input.dir_look.y;

        /*
        if down {
            // TODO: make pretty
            let z = self.zoom.floor() as i32 + 2;
            let scale = 2.0_f64.powi(z as i32);

            let m = Vector2::new(self.input.mouse.x as f64, self.input.mouse.y as f64);
            let w = self.window_size.x as f64;
            let zz = 2.0_f64.powf(self.zoom);

            let px = ((m.x / w - 0.5) / zz + self.offset.x)*scale;
            let py = ((m.y / w - 0.5) / zz + self.offset.y)*scale;

            let p = QuadTreePosition {
                x: px.floor() as u64,
                y: py.floor() as u64,
                z: z as u64,
            };
            if px >= 0.0 && py >= 0.0 && p.x <= p.dim() && p.y <= p.dim() {
                println!("{:?}!", p);
                if let None = self.textures.get_at(p) {
                    let t = mk_texture(&self.sdl.canvas.texture_creator(), p);
                    self.textures.insert_at(p, t);
                    println!("does not exist!");
                }
            }
        }
        */

        if down {
            for i in 0..=1 {
                for j in 0..=1 {
                    let mut p = self.pos.clone();
                    p.child(i, j);
                    let t = mk_texture(&self.sdl.canvas.texture_creator(), p.clone());
                    self.textures.insert_at(&p.path, t);
                }
            }
        }

        let vs = self.textures.at(&self.pos.path).unwrap().values();

        self.sdl.canvas.set_draw_color(Color::RGB(32, 32, 32));
        self.sdl.canvas.clear();

        for (p, v) in &vs {
            let (x, y, z) = p.float_top_left_with_size();
            let w = self.window_size.x as f64;
            let p = (w * x, w * y);
            let s = (w * z, w * z);

            let r = Rect::from((p.0 as i32, p.1 as i32, s.0 as u32, s.1 as u32));
            self.sdl.canvas.copy(v, None, Some(r)).unwrap();

            self.sdl.canvas.set_draw_color(Color::RGB(255, 0, 0));
            self.sdl.canvas.draw_rect(r).unwrap();
        }

        {
            let w = 20;
            let m = self.input.mouse;
            self.sdl.canvas.set_draw_color(Color::RGB(255, 0, 0));
            self.sdl
                .canvas
                .fill_rect(Rect::from_center((m.x, m.y), w, w))
                .unwrap();
        }

        self.sdl.canvas.present();

        quit
    }
}
