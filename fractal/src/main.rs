// TODO: Arbirtrary precision, implementing arbitrary precision is not easy, but we probably want to use this: https://fractalwiki.org/wiki/Perturbation_theory
// TODO: osm tile builder
// TODO: only export a few types to simplify the api
// TODO: wgpu backend
mod atlas;
mod builder_cpu;
mod builder_ocl;
mod fractal;
mod input;
mod math;
mod sdl;
mod state;
mod time;

pub(crate) use self::atlas::AtlasRegion;
pub(crate) use self::atlas::SimpleAtlas;
pub(crate) use self::fractal::builder::TileParams;
pub(crate) use self::fractal::builder::TileType;
pub(crate) use self::fractal::viewport::Viewport;
pub(crate) use self::fractal::TEXTURE_SIZE;
pub(crate) use self::input::Input;
pub(crate) use self::input::InputAction;
pub(crate) use self::input::InputEvent;
pub(crate) use self::math::*;
use serde::{Deserialize, Serialize};
use state::Persist;
use state::Reload;
pub(crate) use tilemap::TilePos;

use self::fractal::viewport::ViewportSave;
use crate::builder_cpu::BuilderCPU;
use crate::builder_ocl::BuilderOCL;
use crate::math::Rect;
use crate::sdl::Sdl;

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

#[derive(Serialize, Deserialize)]
struct SaveState {
    viewport: ViewportSave,
}

pub fn main() {
    let mut sdl = Sdl::new();

    let mut tile_map = TileMap::new();
    let mut config = Config::new();
    let mut viewport = Viewport::new(sdl.output_size());

    let mut builder_ocl = BuilderOCL::new();
    let mut builder_cpu = BuilderCPU::new();

    let persist = Persist::new();
    if let Ok(state) = persist.load("auto") {
        let state: SaveState = state;
        viewport.load(state.viewport);
    }

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

    {
        let state = SaveState {
            viewport: viewport.save(),
        };

        persist.save("auto", &state).unwrap();
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
