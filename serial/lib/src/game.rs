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
use crate::viewport::Viewport;

enum DragState {
    None,
    From(V2),
}

// TODO: implemnt save and load, this will handle some types that dont work with reload.
// For example the btreemap
pub struct State {
    sdl: Sdl,
    input: Input,
    textures: QuadTree<Texture>,

    pos: Viewport,

    drag: DragState,

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
            pos: Viewport::new(),
            input: Input::new(),
            textures: QuadTree::new(),
            drag: DragState::None,
            window_size: Vector2::new(800, 600),
        }
    }

    fn info(&self) {
        let mouse_view = self.screen_to_view(self.input.mouse);
        let mouse_world = self.pos.view_to_world(mouse_view);
        let mouse_view = self.pos.world_to_view(mouse_world);
        let mouse_screen = self.view_to_screen(mouse_view);
        println!(
            "screen  {:6.2} {:6.2}",
            self.input.mouse.x, self.input.mouse.y
        );
        println!("view    {:6.2} {:6.2}", mouse_view.x, mouse_view.y);
        println!("world   {:6.2} {:6.2}", mouse_world.x, mouse_world.y);
        println!("screen2 {:6.2} {:6.2}", mouse_screen.x, mouse_screen.y);
    }

    pub fn update(&mut self) -> bool {
        let mut quit = false;

        let dt = 1.0 / 60.0;

        let mut down = false;

        self.input.begin();
        let events: Vec<_> = self.sdl.event.poll_iter().collect();
        for event in events {
            self.input.handle_sdl(&event);
            match event {
                Event::Quit { .. } => {
                    quit = true;
                },
                Event::KeyDown {
                    keycode: Some(key), ..
                } => match key {
                    Keycode::Q => quit = true,
                    Keycode::C => down = true,
                    Keycode::Tab => self.info(),
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

                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Right,
                    ..
                } => {
                    self.pos.parent();
                },

                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Left,
                    x,
                    y,
                    ..
                } => {
                    /*
                    let sx = x as f32 / self.window_size.x as f32;
                    let sy = y as f32 / self.window_size.x as f32;
                    let p = 2.0*self.pos.screen_to_world(V2::new(sx, sy));
                    self.pos.child(p.x as u8 % 2, p.y as u8 % 2);
                    let t = mk_texture(&self.sdl.canvas.texture_creator(), self.pos.node.clone());
                    self.textures.at(&self.pos.node.path).unwrap().value = Some(t);
                    */
                },

                Event::Window {
                    win_event: WindowEvent::Resized(x, y),
                    ..
                } => {
                    self.window_size.x = (x as u32).max(1);
                    self.window_size.y = (y as u32).max(1);
                },

                _ => {},
            }
        }

        // println!("pos.scale:  {:?}", self.pos.scale);
        // println!("pos.offset: {:?}", self.pos.offset);

        let mouse_in_view = self.screen_to_view(self.input.mouse);
        self.pos
            .zoom_in(0.1 * self.input.scroll as f32, mouse_in_view);

        self.pos.translate(dt * self.input.dir_move);
        self.pos
            .zoom_in(dt * self.input.dir_look.y, V2::new(0.5, 0.5));

        if let DragState::From(p1) = self.drag {
            self.pos.translate(p1 - mouse_in_view);
        }

        self.drag = if self.input.mouse_down {
            DragState::From(mouse_in_view)
        } else {
            DragState::None
        };

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
            let p = self.pos.get_pos();
            let t = mk_texture(&self.sdl.canvas.texture_creator(), p.clone());
            self.textures.insert_at(&p.path, t);
        }

        self.sdl.canvas.set_draw_color(Color::RGB(32, 32, 32));
        self.sdl.canvas.clear();

        let vs = self.textures.values();
        for (p, v) in &vs {
            let r = self.pos_to_rect(p);
            self.sdl.canvas.copy(v, None, Some(r)).unwrap();
            self.sdl.canvas.set_draw_color(Color::RGB(255, 0, 0));
            self.sdl.canvas.draw_rect(r).unwrap();
        }


        {
            let w = 20;

            let mouse_view = self.screen_to_view(self.input.mouse);
            let mouse_world = self.pos.view_to_world(mouse_view);
            let mouse_view = self.pos.world_to_view(mouse_world);
            let mouse_screen = self.view_to_screen(mouse_view);

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

        {
            let r = self.pos_to_rect(&self.pos.get_pos());
            self.sdl.canvas.set_draw_color(Color::RGB(0, 255, 0));
            self.sdl.canvas.draw_rect(r).unwrap();
        }

        self.sdl.canvas.present();

        quit
    }

    fn pos_to_rect(&self, p: &QuadTreePosition) -> Rect {
        let (x, y, z) = p.float_top_left_with_size();
        let p = V2::new(x as f32, y as f32);
        let w = p + V2::new(z as f32, z as f32);
        let p = self.pos.world_to_view(p);
        let p = self.view_to_screen(p);
        let w = self.pos.world_to_view(w);
        let w = self.view_to_screen(w);
        let r = mk_rect(p, w);
        r
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

    let width  = max_x - min_x;
    let height = max_y - min_y;

    let r = Rect::new(min_x, min_y, width as u32, height as u32);
    r
}
