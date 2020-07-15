pub mod builder;
pub mod content;
pub mod queue;
pub mod viewport;

use self::builder::IsTileBuilder;
use self::builder::TileBuilder;
use self::builder::TileParams;
use self::builder::TileParamsSave;
use self::builder::TileType;
use self::content::TileContent;
use self::queue::Queue;
use self::viewport::{Viewport, ViewportSave};
use crate::math::*;
use crate::state::Reload;
use crate::ColorScheme;
use serde::{Deserialize, Serialize};
use tilemap::TileMap;
use tilemap::TilePos;

// We are blending the textures
pub const PADDING: u32 = 1;
pub const TEXTURE_SIZE: usize = 64 * 2;

#[derive(Debug, Copy, Clone)]
pub enum Task {
    Todo,
    Doing,
}

/// Something that can build textures from tile pixels
// TODO: this is very ugly, kindly remove this
pub trait TileTextureProvider {
    type Texture;
    fn alloc(&mut self, pixels_rgba: &[u8]) -> Self::Texture;
    fn free(&mut self, texture: Self::Texture);
}

/// After so many updates, i am not entierly sure what this struct is supposed to become
// TODO: use nano/microserde? it allows for derives, however no enums :(
pub struct Fractal<T> {
    // state
    // NOTE: pos is public, so no need to forward its methods
    pub pos: Viewport,
    pub params: TileParams,
    pub tiles: TileMap<T>,

    clear: bool,
    queue: Queue,
    builder: TileBuilder,
}

impl<T> Fractal<T> {
    pub fn new(size: Vector2<u32>) -> Self {
        let params = TileParams {
            kind: TileType::Mandelbrot,
            iterations: 64,
            resolution: TEXTURE_SIZE as u32,
            padding: PADDING,
            color: ColorScheme::new(),
        };

        let queue = Queue::new(params.clone());
        let builder = TileBuilder::new(queue.handle());
        Fractal {
            queue,
            builder,
            tiles: TileMap::new(),
            pos: Viewport::new(size),
            clear: false,
            params,
        }
    }

    pub fn add_builder<B: IsTileBuilder + Send + 'static>(&mut self, b: B) {
        self.builder.add_builder(b);
    }

    pub fn reload(&mut self) {
        self.clear = true;
    }

    pub fn update_tiles(&mut self, texture_creator: &mut impl TileTextureProvider<Texture = T>) {
        if self.clear {
            self.queue.set_params(&self.params);
            let tiles = std::mem::replace(&mut self.tiles, TileMap::new());
            for (_, t) in tiles.into_iter() {
                texture_creator.free(t);
            }
            self.clear = false;
        }

        self.queue.update(&self.pos);

        // read from builders
        while let Ok(r) = self.queue.try_recv() {
            if r.version != self.queue.params_version {
                continue;
            }

            let t = texture_creator.alloc(&r.content.pixels);
            self.tiles.insert(r.pos, t);
        }

        // Free textures
        let new_iter = self.pos.get_pos_all();
        self.tiles
            .update_with(new_iter, |_, v| texture_creator.free(v), |_| None);
    }

    pub fn debug_queue_tiles(&mut self) -> Vec<(TilePos, Task)> {
        self.queue
            .tiles
            .lock_high()
            .map
            .iter()
            .map(|(a, b)| (*a, *b))
            .collect()
    }
}

#[derive(Serialize, Deserialize)]
pub struct FractalSave {
    pub pos: ViewportSave,
    pub params: TileParamsSave,
}

impl<T> Reload for Fractal<T> {
    type Storage = FractalSave;

    fn load(&mut self, data: Self::Storage) {
        self.pos.load(data.pos);
        self.params.load(data.params);
        self.clear = true;
    }

    fn save(&self) -> Self::Storage {
        FractalSave {
            pos: self.pos.save(),
            params: self.params.save(),
        }
    }
}
