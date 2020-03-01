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

    pos: Viewport,

    offset: Vector2<f32>,
    zoom: f32,

    window_size: Vector2<u32>,
}

struct Viewport {
    node: QuadTreePosition,

    scale: f32,
    offset: Vector2<f32>,
}

impl Viewport {
    fn new() -> Self {
        Viewport { node: QuadTreePosition::root(), scale: 1., offset: V2::zero() }
    }

    fn world_to_view(&self, p: V2) -> V2 {
        let (x, y, s) = self.node.float_top_left_with_size();
        let x = x as f32;
        let y = y as f32;
        let s = s as f32;
        V2::new((p.x - self.offset.x) / self.scale, (p.y - self.offset.y) / self.scale)
    }

    fn view_to_world(&self, p: V2) -> V2 {
        let (x, y, s) = self.node.float_top_left_with_size();
        let x = x as f32;
        let y = y as f32;
        let s = s as f32;
        V2::new((p.x) * self.scale + self.offset.x, (p.y) * self.scale + self.offset.y)
    }

    fn child(&mut self, i: u8, j: u8) {
        self.node.child(i, j);
    }

    fn parent(&mut self) {
        self.node.parent();
    }
}

fn mk_texture<T>(canvas: &TextureCreator<T>, p: QuadTreePosition) -> Texture {
    let size = 256;
    let mut texture = canvas.create_texture_static(PixelFormatEnum::RGBA8888, size, size).unwrap();
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
            pos: Viewport::new(),
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
                },
                Event::KeyDown { keycode: Some(key), .. } => match key {
                    Keycode::Q => quit = true,
                    Keycode::C => down = true,
                    Keycode::R => {
                        self.textures.reduce_to(1);
                    },
                    Keycode::F => self.textures.clear(),
                    Keycode::Num0 => self.pos.parent(),
                    Keycode::Num1 => self.pos.child(0, 0),
                    Keycode::Num2 => self.pos.child(1, 0),
                    Keycode::Num3 => self.pos.child(0, 1),
                    Keycode::Num4 => self.pos.child(1, 1),
                    _ => (),
                },

                Event::MouseButtonDown { mouse_btn: MouseButton::Right, .. } => {
                    self.pos.parent();
                },

                Event::MouseButtonDown { mouse_btn: MouseButton::Left, x, y, .. } => {
                    /*
                    let sx = x as f32 / self.window_size.x as f32;
                    let sy = y as f32 / self.window_size.x as f32;
                    let p = 2.0*self.pos.screen_to_world(V2::new(sx, sy));
                    self.pos.child(p.x as u8 % 2, p.y as u8 % 2);
                    let t = mk_texture(&self.sdl.canvas.texture_creator(), self.pos.node.clone());
                    self.textures.at(&self.pos.node.path).unwrap().value = Some(t);
                    */
                },

                Event::MouseWheel { y, .. } => {
                    self.zoom += 0.5 * (y as f32);
                },

                Event::Window { win_event: WindowEvent::Resized(x, y), .. } => {
                    self.window_size.x = (x as u32).max(1);
                    self.window_size.y = (y as u32).max(1);
                },

                _ => {},
            }
        }

        self.pos.offset += dt * self.input.dir_move * self.pos.scale;

        let p = self.screen_to_view(self.input.mouse);
        self.pos.offset += self.pos.scale * p;
        self.pos.scale *= 1.0 + dt * self.input.dir_look.y * 1.0;
        self.pos.offset -= self.pos.scale * p;

        println!("pos.scale:  {:?}", self.pos.scale);
        println!("pos.offset: {:?}", self.pos.offset);

        // if down {
        // TODO: make pretty
        // let z = self.zoom.floor() as i32 + 2;
        // let scale = 2.0_f32.powi(z as i32);
        //
        // let m = Vector2::new(self.input.mouse.x as f32, self.input.mouse.y as f32);
        // let w = self.window_size.x as f32;
        // let zz = 2.0_f32.powf(self.zoom);
        //
        // let px = ((m.x / w - 0.5) / zz + self.offset.x)*scale;
        // let py = ((m.y / w - 0.5) / zz + self.offset.y)*scale;
        //
        // let p = QuadTreePosition {
        // x: px.floor() as u64,
        // y: py.floor() as u64,
        // z: z as u64,
        // };
        // if px >= 0.0 && py >= 0.0 && p.x <= p.dim() && p.y <= p.dim() {
        // println!("{:?}!", p);
        // if let None = self.textures.get_at(p) {
        // let t = mk_texture(&self.sdl.canvas.texture_creator(), p);
        // self.textures.insert_at(p, t);
        // println!("does not exist!");
        // }
        // }
        // }

        if down {
            for i in 0..=1 {
                for j in 0..=1 {
                    let mut p = self.pos.node.clone();
                    p.child(i, j);
                    let t = mk_texture(&self.sdl.canvas.texture_creator(), p.clone());
                    self.textures.insert_at(&p.path, t);
                }
            }
        }

        let vs = self.textures.values();

        self.sdl.canvas.set_draw_color(Color::RGB(32, 32, 32));
        self.sdl.canvas.clear();

        println!("6");

        for (p, v) in &vs {
            let (x, y, z) = p.float_top_left_with_size();
            let w = self.window_size.x as f32;

            let x = x as f32;
            let y = y as f32;
            let z = z as f32;

            let p = (V2::new(x, y));

            let p = (w * p.x, w * p.y);
            let s = (w * z, w * z);

            let r = Rect::from((p.0 as i32, p.1 as i32, s.0 as u32, s.1 as u32));
            self.sdl.canvas.copy(v, None, Some(r)).unwrap();

            self.sdl.canvas.set_draw_color(Color::RGB(255, 0, 0));
            self.sdl.canvas.draw_rect(r).unwrap();
        }

        {
            let w = 20;

            let mouse_view = self.screen_to_view(self.input.mouse);
            println!("view   {:6.2} {:6.2}", mouse_view.x, mouse_view.y);
            let mouse_world = self.pos.view_to_world(mouse_view);
            let mouse_view = self.pos.world_to_view(mouse_world);
            let mouse_screen = self.view_to_screen(mouse_view);

            println!("screen {:6.2} {:6.2}", mouse_screen.x, mouse_screen.y);
            println!("view   {:6.2} {:6.2}", mouse_view.x, mouse_view.y);
            println!("world  {:6.2} {:6.2}", mouse_world.x, mouse_world.y);
            self.sdl.canvas.set_draw_color(Color::RGB(255, 0, 0));
            self.sdl
                .canvas
                .fill_rect(Rect::from_center((mouse_screen.x, mouse_screen.y), w, w))
                .unwrap();

            // word space
            let p_min = V2::new(0., 0.);
            let p_max = V2::new(1., 1.);

            let p_min = self.pos.world_to_view(p_min);
            let p_min = self.view_to_screen(p_min);

            let p_max = self.pos.world_to_view(p_max);
            let p_max = self.view_to_screen(p_max);

            self.sdl.canvas.set_draw_color(Color::RGB(255, 0, 0));
            self.sdl.canvas.draw_rect(mk_rect(p_min, p_max)).unwrap();
        }

        self.sdl.canvas.present();

        quit
    }

    fn screen_to_view(&self, p: V2i) -> V2 {
        V2::new(
            p.x as f32 / self.window_size.x as f32,
            1.0 - p.y as f32 / self.window_size.y as f32,
        )
    }

    fn view_to_screen(&self, p: V2) -> V2i {
        V2i::new(
            (p.x * self.window_size.x as f32) as i32,
            ((1.0 - p.y) * self.window_size.y as f32) as i32,
        )
    }
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
