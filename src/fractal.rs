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
use self::viewport::Viewport;
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

pub type TaskMap = TileMap<Task<TileContent>>;

pub fn fn_true() -> bool {
    true
}

/// After so many updates, i am not entierly sure what this struct is supposed to become
// TODO: use microserde? but we need derives
#[derive(Serialize, Deserialize)]
pub struct Fractal<T> {
    // state
    // NOTE: pos is public, so no need to forward its methods
    pub pos: Viewport,
    pub params: TileParams,

    #[serde(skip, default = "fn_true")]
    pub clear: bool,

    // this uses a workaround to prevent incorrect `T: Default` bounds.
    // see: https://github.com/serde-rs/serde/issues/1541
    // TODO: maybe go back to locks?, i want to be able to clear a channel, that is not possible
    // as far as i know, also we have to be able to select when to recieve a position
    // TODO: params contain a version number
    // NOTE: These are rendered tiles
    #[serde(skip, default = "TileMap::new")]
    pub tiles: TileMap<T>,

    #[serde(skip, default = "Builder::new")]
    pub builder: Builder,
}

impl<T> Fractal<T> {
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
        self.builder.queue.set_params(&self.params);
    }

    pub fn update_tiles(&mut self, texture_creator: &mut impl TileTextureProvider<Texture = T>) {
        if self.clear {
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
