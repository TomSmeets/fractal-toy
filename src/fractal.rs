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
use crate::tilemap::Task;
use crate::tilemap::TileMap;
use crate::ColorScheme;

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

/// After so many updates, i am not entierly sure what this struct is supposed to become
#[derive(Serialize, Deserialize)]
pub struct Fractal<T> {
    // state
    // NOTE: pos is public, so no need to forward its methods
    pub pos: Viewport,
    pub params: TileParams,

    // this uses a workaround to prevent incorrect `T: Default` bounds.
    // see: https://github.com/serde-rs/serde/issues/1541
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
            params: TileParams {
                kind: TileType::Mandelbrot,
                iterations: 64,
                padding: 1,
                resolution: TEXTURE_SIZE as u32,
                color: ColorScheme::new(),
            },
        }
    }

    pub fn reload(&mut self) {
        self.tiles.clear();
    }

    pub fn update_tiles(&mut self, texture_creator: &mut impl TileTextureProvider<Texture = T>) {
        let queue = &mut self.builder;
        queue.builder.update();

        // send todo to builders
        for (r, t) in self.tiles.iter_mut() {
            if let Task::Todo = t {
                if let Ok(_) = queue.queue.try_send(self.params.clone(), *r) {
                    *t = Task::Doing;
                } else {
                    break;
                }
            }
        }

        // read from builders
        while let Ok(r) = queue.queue.try_recv(&self.params) {
            if let Some((p, t)) = r {
                if let Some(v) = self.tiles.get_mut(&p) {
                    let t = texture_creator.alloc(&t.pixels);
                    if let Task::Doing = v {
                        *v = Task::Done(t);
                    }
                }
            }
        }

        let new_iter = self.pos.get_pos_all();
        self.tiles
            .update_with(new_iter, |_, v| texture_creator.free(v));
    }
}
