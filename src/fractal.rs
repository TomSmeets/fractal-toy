use crate::math::*;
use serde::{Deserialize, Serialize};

mod builder;
mod content;
mod queue;
mod viewport;

pub use self::content::TileContent;

use self::builder::TileBuilder;
pub use self::builder::TileType;
use self::builder::{TileParams, TileParamsSave};
use self::queue::Queue;
use self::viewport::{Viewport, ViewportSave};
use crate::state::Reload;
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
    pub queue: Queue,
    pub builder: TileBuilder,
}

impl<T> Fractal<T> {
    pub fn new(size: Vector2<u32>) -> Self {
        let queue = Queue::new();
        let builder = TileBuilder::new(queue.handle());
        Fractal {
            queue,
            builder,
            tiles: TileMap::new(),
            pos: Viewport::new(size),
            clear: false,
            params: TileParams::default(),
        }
    }

    pub fn reload(&mut self) {
        self.clear = true;
    }

    pub fn update_tiles(&mut self, texture_creator: &mut impl TileTextureProvider<Texture = T>) {
        if self.clear {
            self.queue.set_params(&self.params);
            let tiles = std::mem::replace(&mut self.tiles, TileMap::new());
            for (_, t) in tiles.tiles.into_iter() {
                texture_creator.free(t);
            }
            self.clear = false;
        }

        // blocking
        let version = self.queue.update(&self.pos);

        // read from builders
        while let Ok(r) = self.queue.try_recv() {
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
