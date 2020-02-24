use sdl2::pixels::*;
use sdl2::event::*;
use sdl2::keyboard::Keycode;
use sdl2::rect::*;
use sdl2::render::*;

use std::collections::*;
use std::hash::*;

use crate::math::*;
use crate::input::*;
use crate::fractal::*;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[derive(Clone)]
pub struct TilePos {
    x: i64,
    y: i64,
    z: i32,
}


pub struct Sdl {
    ctx:   sdl2::Sdl,
    event: sdl2::EventPump,
    canvas: Canvas<sdl2::video::Window>,

}

impl Sdl {
    fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem.window("rust-sdl2 demo", 800, 600)
            .position_centered()
            .build()
            .unwrap();

        let event = sdl_context.event_pump().unwrap();
        let canvas = window.into_canvas().present_vsync().build().unwrap();

        Sdl {
            ctx: sdl_context,
            event: event,
            canvas: canvas,
        }
    }
}

// TODO: implemnt save and load, this will handle some types that dont work with reload.
// For example the btreemap
pub struct State {
    sdl: Sdl,
    message: String,
    input: Input,
    textures: HashMap<TilePos, Texture>,

    offset: Vector2<f64>,
    zoom: f64,

    window_size: Vector2<u32>,
}

fn mk_texture<T>(canvas: &TextureCreator<T>, p: TilePos) -> Texture {
    let size = 256;
    let mut texture = canvas.create_texture_static(PixelFormatEnum::RGBA8888, size, size).unwrap();
    let mut pixels = vec![0; (size*size*4) as usize];
    draw_tile(&mut pixels, p.x, p.y, p.z);
    texture.update(None, &pixels, (4*size) as usize).unwrap();
    texture
}


impl State {
    pub fn unload(&mut self) { }
    pub fn reload(&mut self) { }

    pub fn new() -> State {

        // let sz = Vector2::new(64, 64);
        // let texture = canvas.create_texture_static(PixelFormatEnum::RGBA8888, sz.x, sz.y).unwrap();

        State {
            sdl: Sdl::new(),
            message: "Hello".to_string(),
            input: Input::new(),
            textures: HashMap::new(),
            offset:   Vector2::zero(),
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
                Event::Quit {..} => { quit = true; }
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => { quit = true; },
                Event::KeyDown { keycode: Some(Keycode::C), .. } => { down = true; },
                Event::KeyDown { keycode: Some(Keycode::F), .. } => {
                    self.textures.clear();
                },

                Event::MouseWheel { y, .. } => {
                    self.zoom    += 0.5*(y as f64);
                    self.message = "WHEEL!".to_owned();
                },

                Event::Window { win_event: WindowEvent::Resized(x, y), .. } => {
                    self.message = "resize!".to_owned();
                    self.window_size.x = (x as u32).max(1);
                    self.window_size.y = (y as u32).max(1);
                },

                _ => {}
            }
        }


        self.offset += dt*self.input.dir_move*0.5_f64.powf(self.zoom);
        self.zoom   += 2.0*dt*self.input.dir_look.y;


        println!("{:?}", self.offset);

        // self.textures = BTreeMap::new();


        if down {
            // TODO: make pretty
            let z = self.zoom.floor() as i32 + 2;
            let scale = 2.0_f64.powi(z);

            let m = Vector2::new(self.input.mouse.x as f64, self.input.mouse.y as f64);
            let w = self.window_size.x as f64;
            let zz = 2.0_f64.powf(self.zoom);

            let px = ((m.x / w - 0.5) / zz + self.offset.x) * scale;
            let py = ((m.y / w - 0.5) / zz + self.offset.y) * scale;

            let p = TilePos { x: px.floor() as i64, y: py.floor() as i64, z: z};
            println!("{:?}!", p);
            if !self.textures.contains_key(&p){
                let t = mk_texture(&self.sdl.canvas.texture_creator(), p.clone());
                self.textures.insert(p, t);
                println!("does not exist!");
            }

        }

        // doit(1,  0,  1);
        // doit(4,  0,  2);
        // doit(0,  0,  2);
        // doit(0,  0,  2);

        // if let Some((w_w, w_h)) = resize {

        //     let w = 128*4;
        //     let h = 128*4;

        //     let mut pixels = vec![0; (w*h*4) as usize];
        //     //  draw_mandel(&mut pixels, self.texture_size.x, self.texture_size.y, self.zoom, self.offset);
        //     // draw_tile(&mut pixels, 0, 1<<30, 31);

        //     let z: i32 = 1;
        //     let x: f64 = 0.0;
        //     let y: f64 = 1.0;
        //     let scale = 2f64.powi(z);
        //     draw_tile(&mut pixels, (x*scale).round() as i64, (y*scale).round() as i64, z);
        // }
        //

        self.sdl.canvas.set_draw_color(Color::RGB(255, 255, 0));
        self.sdl.canvas.clear();

        // println!("texture count: {:?}", self.textures.len());
        let w = self.window_size.x as f64;
        let z = 2.0_f64.powf(self.zoom);


        let mut ts : Vec<(&TilePos, &Texture)> = self.textures.iter().collect();
        // TODO: make fast, quadtree?
        ts.sort_by(|(a, _), (b, _)| { a.z.cmp(&b.z) });
        for (k, v) in ts {
            let scale = 0.5_f64.powi(k.z);

            let x = (k.x as f64)*scale - self.offset.x as f64;
            let y = (k.y as f64)*scale - self.offset.y as f64;

            self.sdl.canvas.copy(v, None, Some((
                (w*(z*x + 0.5)).floor()    as i32, (w*(z*y + 0.5)).floor()    as i32,
                (w*z*scale).ceil() as u32, (w*z*scale).ceil() as u32
            ).into())).unwrap();
        }

        {
            let w = 20;
            let m = self.input.mouse;
            self.sdl.canvas.set_draw_color(Color::RGB(255, 0, 0));
            self.sdl.canvas.fill_rect(Rect::from_center((m.x, m.y), w, w)).unwrap();
        }

        self.sdl.canvas.present();
        return quit;
    }
}
