use crate::math::*;
use crate::module::time::DeltaTime;
use crate::module::Input;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

pub mod builder;
pub mod storage;
pub mod tile;
pub mod viewport;

use self::builder::queue::{TileQueue, WorkQueue};
use self::builder::TileBuilder;
use self::builder::TileParams;
use self::builder::TileType;
use self::storage::TileStorage;
use self::viewport::Viewport;

// We are blending the textures
pub const PADDING: u32 = 1;
pub const TEXTURE_SIZE: usize = 64 * 2;

/// Something that can build textures from tile pixels
pub trait TileTextureProvider {
    type Texture;
    fn alloc(&mut self, pixels_rgba: &[u8]) -> Self::Texture;
    fn free(&mut self, texture: Self::Texture);
}

/// After so many updates, i am not entierly sure what this struct is supposed to become
#[derive(Serialize, Deserialize)]
pub struct Fractal<T> {
    // state
    pub pos: Viewport,
    pub params: TileParams,

    // this uses a workaround to prevent incorrect `T: Default` bounds.
    // see: https://github.com/serde-rs/serde/issues/1541
    #[serde(skip, default = "TileStorage::new")]
    pub tiles: TileStorage<T>,

    #[serde(skip)]
    pub queue: Arc<Mutex<TileQueue>>,
    #[serde(skip)]
    tile_builder: Option<TileBuilder>,
}

impl<T> Fractal<T> {
    pub fn new(size: Vector2<u32>) -> Self {
        Fractal {
            tiles: TileStorage::new(),
            pos: Viewport::new(size),
            tile_builder: None,
            queue: Arc::new(Mutex::new(WorkQueue::new())),

            params: TileParams {
                kind: TileType::Mandelbrot,
                iterations: 64,
            },
        }
    }

    pub fn update_tiles(&mut self, texture_creator: &mut impl TileTextureProvider<Texture = T>) {
        // This recreates tile builders when entire struct is deserialized
        if self.tile_builder.is_none() {
            self.tile_builder = Some(TileBuilder::new(Arc::clone(&self.queue)));
        }

        self.tile_builder.as_mut().unwrap().update();

        let mut queue = match self.queue.try_lock() {
            Err(_) => return,
            Ok(q) => q,
        };

        self.tiles
            .update_tiles(&mut queue, self.params, &self.pos, texture_creator);
    }

    pub fn do_input(&mut self, input: &Input, dt: DeltaTime) {
        if input.scroll != 0 {
            self.pos.zoom_in_at(0.3 * input.scroll as f64, input.mouse);
        }

        self.pos.translate({
            let mut p = dt.0 as f64 * input.dir_move * 2.0 * self.pos.size_in_pixels.x;
            p.y *= -1.0;
            to_v2i(p)
        });
        self.pos.zoom_in(dt.0 as f64 * input.zoom as f64 * 3.5);
        self.pos.translate(-input.drag);

        // TODO: in the future we want some kind of ui, or cli interface
        if input.iter_inc {
            self.params.iterations += 40;
        }

        if input.iter_dec {
            self.params.iterations -= 40;
            self.params.iterations = self.params.iterations.max(3);
        }

        if input.cycle {
            self.params.kind = self.params.kind.next();
        }
    }
}
