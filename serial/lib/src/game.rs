use sdl2::event::*;
use sdl2::keyboard::Keycode;
use sdl2::pixels::*;
use sdl2::rect::*;
use sdl2::render::*;
use sdl2::video::*;

use std::hash::*;

use crate::fractal::*;
use crate::input::*;
use crate::math::*;
use crate::quadtree::*;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone)]
pub struct TilePos {
	x: i64,
	y: i64,
	z: i32,
}

pub struct Sdl {
	ctx: sdl2::Sdl,
	video: sdl2::VideoSubsystem,
	event: sdl2::EventPump,
	canvas: Canvas<Window>,
}

impl Sdl {
	fn new() -> Self {
		let ctx = sdl2::init().unwrap();
		let video = ctx.video().unwrap();

		let window = video.window("rust-sdl2 demo", 800, 600).position_centered().build().unwrap();

		let event = ctx.event_pump().unwrap();
		let canvas = window.into_canvas().present_vsync().build().unwrap();

		Sdl { ctx, video, event, canvas }
	}
}

// TODO: implemnt save and load, this will handle some types that dont work with reload.
// For example the btreemap
pub struct State {
	sdl: Sdl,
	message: String,
	input: Input,
	textures: QuadTree<Texture>,

	offset: Vector2<f64>,
	zoom: f64,

	window_size: Vector2<u32>,
}

fn mk_texture<T>(canvas: &TextureCreator<T>, p: QuadTreePosition) -> Texture {
	let size = 256;
	let mut texture = canvas.create_texture_static(PixelFormatEnum::RGBA8888, size, size).unwrap();
	let mut pixels = vec![0; (size * size * 4) as usize];
	draw_tile(&mut pixels, p.x as i64, p.y as i64, p.z as i32);

	texture.update(None, &pixels, (4 * size) as usize).unwrap();
	texture
}

impl State {
	pub fn unload(&mut self) {}
	pub fn reload(&mut self) {}

	pub fn new() -> State {
		let sdl = Sdl::new();

		// TODO: get window size
		State {
			sdl: sdl,
			message: "Hello".to_string(),
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
				Event::KeyDown { keycode: Some(Keycode::Q), .. } => {
					quit = true;
				}
				Event::KeyDown { keycode: Some(Keycode::C), .. } => {
					down = true;
				}

				Event::KeyDown { keycode: Some(Keycode::R), .. } => {
					let z = self.zoom.floor() as i32 + 2;
					self.textures.reduce_to(z);
				}
				Event::KeyDown { keycode: Some(Keycode::F), .. } => {
					self.textures.clear();
				}

				Event::MouseWheel { y, .. } => {
					self.zoom += 0.5 * (y as f64);
					self.message = "WHEEL!".to_owned();
				}

				Event::Window {
					win_event: WindowEvent::Resized(x, y),
					..
				} => {
					self.message = "resize!".to_owned();
					self.window_size.x = (x as u32).max(1);
					self.window_size.y = (y as u32).max(1);
				}

				_ => {}
			}
		}

		self.offset += dt * self.input.dir_move * 0.5_f64.powf(self.zoom);
		self.zoom += 2.0 * dt * self.input.dir_look.y;

		if down {
			// TODO: make pretty
			let z = self.zoom.floor() as i32 + 2;
			let scale = 2.0_f64.powi(z as i32);

			let m = Vector2::new(self.input.mouse.x as f64, self.input.mouse.y as f64);
			let w = self.window_size.x as f64;
			let zz = 2.0_f64.powf(self.zoom);

			let px = ((m.x / w - 0.5) / zz + self.offset.x) * scale;
			let py = ((m.y / w - 0.5) / zz + self.offset.y) * scale;

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

		{
			// Insert some tiles at known positions for testing purposes
			let ps = [QuadTreePosition { x: 0, y: 0, z: 1 }, QuadTreePosition { x: 1, y: 0, z: 1 }, QuadTreePosition { x: 1, y: 1, z: 1 }];

			for p in ps.iter() {
				if let None = self.textures.get_at(p.clone()) {
					let t = mk_texture(&self.sdl.canvas.texture_creator(), p.clone());
					self.textures.insert_at(p.clone(), t);
				}
			}
		}

		self.sdl.canvas.set_draw_color(Color::RGB(255, 255, 0));
		self.sdl.canvas.clear();

		// println!("texture count: {:?}", self.textures.len());
		let w = self.window_size.x as f64;
		let z = 2.0_f64.powf(self.zoom);

		let ts = self.textures.values();
		for (k, v) in ts {
			let scale = 0.5_f64.powi(k.z as i32);

			let x = (k.x as f64) * scale - self.offset.x as f64;
			let y = (k.y as f64) * scale - self.offset.y as f64;

			self.sdl
				.canvas
				.copy(
					v,
					None,
					Some(
						(
							(w * (z * x + 0.5)).floor() as i32,
							(w * (z * y + 0.5)).floor() as i32,
							(w * z * scale).ceil() as u32,
							(w * z * scale).ceil() as u32,
						)
							.into(),
					),
				)
				.unwrap();
		}

		{
			let w = 20;
			let m = self.input.mouse;
			self.sdl.canvas.set_draw_color(Color::RGB(255, 0, 0));
			self.sdl.canvas.fill_rect(Rect::from_center((m.x, m.y), w, w)).unwrap();
		}

		self.sdl.canvas.present();

		quit
	}
}
