use crate::math::*;
use serde::{Deserialize, Serialize};

mod builder;
mod content;
mod queue;
mod viewport;

pub use self::content::TileContent;

use self::builder::TileBuilder;
use self::builder::TileParams;
pub use self::builder::TileType;
use self::queue::Queue;
use self::viewport::{Viewport, ViewportSave};
use tilemap::Task;
use tilemap::TileMap;

// We are blending the textures
pub const PADDING: u32 = 1;
pub const TEXTURE_SIZE: usize = 64 * 2;

/// Something that can build textures from tile pixels
// TODO: this is very ugly, kindly remove this
pub trait TileTextureProvider {
    type Texture;
    fn alloc(&mut self, pixels_rgba: &[u8]) -> Self::Texture;
    fn free(&mut self, texture: Self::Texture);
}

// TODO: uuugh anothher stuct named `Builder`, rename it or whatever
pub struct Builder {
    pub queue: Queue,
    pub builder: TileBuilder,
}

impl Builder {
    pub fn new() -> Self {
        let queue = Queue::new();
        let builder = TileBuilder::new(queue.handle());
        Builder { queue, builder }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

pub type TaskMap = TileMap<Task<TileContent>>;

pub fn fn_true() -> bool {
    true
}

/// After so many updates, i am not entierly sure what this struct is supposed to become
// TODO: use nano/microserde? it allows for derives, however no enums :(
pub struct Fractal<T> {
    // state
    // NOTE: pos is public, so no need to forward its methods
    pub pos: Viewport,
    pub params: TileParams,
    pub clear: bool,
    pub tiles: TileMap<T>,
    pub builder: Builder,
}

#[derive(Serialize, Deserialize)]
pub struct FractalSave {
    pub pos: ViewportSave,
    pub params: TileParams,
}

impl<T> Fractal<T> {
    pub fn load(&mut self, data: FractalSave) {
        self.pos.load(data.pos);
        self.params = data.params;
        self.clear = true;
    }

    pub fn save(&self) -> FractalSave {
        FractalSave {
            pos: self.pos.save(),
            params: self.params.clone(),
        }
    }

    pub fn new(size: Vector2<u32>) -> Self {
        Fractal {
            tiles: TileMap::new(),
            pos: Viewport::new(size),
            builder: Builder::new(),
            clear: false,
            params: TileParams::default(),
        }
    }

    pub fn reload(&mut self) {
        self.clear = true;
    }

    pub fn update_tiles(&mut self, texture_creator: &mut impl TileTextureProvider<Texture = T>) {
        if self.clear {
            self.builder.queue.set_params(&self.params);
            let tiles = std::mem::replace(&mut self.tiles, TileMap::new());
            for (_, t) in tiles.tiles.into_iter() {
                texture_creator.free(t);
            }
            self.clear = false;
        }

        // blocking
        let version = self.builder.queue.update(&self.pos);

        // read from builders
        while let Ok(r) = self.builder.queue.try_recv() {
            if r.version != version {
                continue;
            }

            let t = texture_creator.alloc(&r.content.pixels);
            self.tiles.tiles.insert(r.pos, t);
        }

        // Free textures
        let new_iter = self.pos.get_pos_all();
        self.tiles
            .update_with(new_iter, |_, v| texture_creator.free(v), |_| None);
    }
}
