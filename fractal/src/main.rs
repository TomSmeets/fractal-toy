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

use self::atlas::AtlasRegion;
use self::atlas::SimpleAtlas;
use self::fractal::builder::TileParams;
use self::fractal::builder::TileType;
use self::fractal::viewport::Viewport;
use self::fractal::viewport::ViewportSave;
use self::input::Input;
use self::input::InputAction;
use self::input::InputEvent;
use self::math::*;
use crate::builder_cpu::BuilderCPU;
use crate::builder_ocl::BuilderOCL;
use crate::math::Rect;
use crate::sdl::Sdl;
use serde::{Deserialize, Serialize};
use state::Persist;
use state::Reload;
use tilemap::TilePos;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct TextureSizeAndPadding {
    pub size: u32,
    pub padding: u32,
}

impl TextureSizeAndPadding {
    pub fn stride_in_bytes(self) -> usize {
        self.size as usize * 4
    }
    pub fn size_in_bytes(self) -> usize {
        self.size as usize * self.size as usize * 4
    }
}

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
    debug: bool,
    params: TileParams,
}

pub struct TileMap {
    pub tiles: tilemap::TileMap<Tile>,
}

impl Config {
    fn new() -> Self {
        Self {
            changed: true,
            debug: false,
            params: TileParams {
                kind: TileType::Mandelbrot,
                iterations: 64,
                size: TextureSizeAndPadding {
                    size: 64 * 2,
                    padding: 1,
                },
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
struct SaveState {
    viewport: Option<ViewportSave>,
    debug: bool,
}

pub fn main() {
    // Configure
    let mut config = Config::new();

    // Init
    let mut tile_map = TileMap::new();
    let mut sdl = Sdl::new(&config);
    let mut viewport = Viewport::new(sdl.output_size());

    let mut builder_ocl = BuilderOCL::new();
    let mut builder_cpu = BuilderCPU::new();

    // load saved state
    let persist = Persist::new();
    if let Ok(state) = persist.load("auto") {
        let state: SaveState = state;
        if let Some(s) = state.viewport {
            viewport.load(s)
        }
        config.debug = state.debug;
    }

    use std::time::Instant;
    let mut start_time = Instant::now();
    loop {
        let dt = {
            let end_time = Instant::now();
            let dt = end_time - start_time;
            println!("dt: {:?}", dt);
            start_time = end_time;
            dt.as_secs_f32()
        };

        let input = sdl.events();
        input.move_viewport(dt, &mut viewport);
        input.update_config(&mut config);

        if input.is_quit() {
            break;
        }

        tile_map.update(&config, &viewport);
        builder_ocl.update(&config, &mut tile_map);
        builder_cpu.update(&config, &mut tile_map);

        sdl.render(&config.params, &tile_map, &viewport);

        config.changed = false;
    }

    {
        let state = SaveState {
            viewport: Some(viewport.save()),
            debug: config.debug,
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

    fn update(&mut self, config: &Config, vp: &Viewport) {
        // Free textures
        let new_iter = vp.get_pos_all(config.params.size).map(|x| (x, ()));
        self.tiles
            .update_with(new_iter, |_, _| (), |_, _| Some(Tile::Todo));
    }
}
