mod builder_cpu;
mod sdl;

use crate::builder_cpu::BuilderCPU;
use crate::sdl::Sdl;
use crossbeam_channel::{Receiver, Sender};
use fractal_toy::math::Rect;
use fractal_toy::IsTileBuilder;
use fractal_toy::TileParams;
use fractal_toy::TilePos;
use fractal_toy::TileType;
use fractal_toy::Viewport;
use fractal_toy::TEXTURE_SIZE;
use std::thread::JoinHandle;

pub fn rect_to_sdl(r: Rect) -> sdl2::rect::Rect {
    sdl2::rect::Rect::new(r.pos.x, r.pos.y, r.size.x as u32, r.size.y as u32)
}

pub enum Tile {
    Todo,
    Doing,
    Done(Vec<u8>),
    Used,
}

pub struct Config {
    changed: bool,
    params: TileParams,
}

pub struct TileMap {
    pub tiles: tilemap::TileMap<Tile>,
}
struct BuilderOCL {}

impl Config {
    fn new() -> Self {
        Self {
            changed: true,
            params: TileParams {
                kind: TileType::Mandelbrot,
                iterations: 64,
                resolution: TEXTURE_SIZE as u32,
                padding: 1,
            },
        }
    }
}

impl BuilderOCL {
    fn new() -> Self {
        Self {}
    }
    pub fn update(&mut self, config: &Config, map: &mut TileMap) {
        // for (_, t) in map.tiles.iter_mut() {
        //     if let Tile::Todo = t {
        //         *t = Tile::Done(vec![0, 0, 255, 0]);
        //         break;
        //     }
        // }
    }
}

pub fn main() {
    let mut sdl = Sdl::new();

    let mut tile_map = TileMap::new();
    let mut config = Config::new();
    let mut viewport = Viewport::new(sdl.output_size());

    let mut builder_ocl = BuilderOCL::new();
    let mut builder_cpu = BuilderCPU::new();

    use std::time::Instant;
    let mut start_time = Instant::now();
    loop {
        let input = sdl.events();
        input.move_viewport(&mut viewport);
        input.update_config(&mut config);

        if input.is_quit() {
            break;
        }

        tile_map.update(&viewport);

        builder_ocl.update(&config, &mut tile_map);
        builder_cpu.update(&config, &mut tile_map);

        sdl.render(&tile_map, &viewport);

        config.changed = false;

        let end_time = Instant::now();
        let dt = end_time - start_time;
        println!("dt: {:?}", dt);
        start_time = end_time;
    }
}

impl TileMap {
    fn new() -> Self {
        Self {
            tiles: tilemap::TileMap::new(),
        }
    }

    fn update(&mut self, vp: &Viewport) {
        // Free textures
        let new_iter = vp.get_pos_all().map(|x| (x, ()));
        self.tiles
            .update_with(new_iter, |_, _| (), |_, _| Some(Tile::Todo));
    }
}
