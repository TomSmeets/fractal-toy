use crate::iter::compare::{CompareIter, ComparedValue};
use crate::math::*;
use crate::module::time::DeltaTime;
use crate::module::{input::InputAction, Input};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

pub mod builder;
pub mod storage;
pub mod tile;
pub mod viewport;

use self::builder::queue::{TileQueue, WorkQueue};
use self::builder::TileBuilder;
use self::builder::TileType;
use self::storage::TileStorage;
use self::viewport::Viewport;

pub const TEXTURE_SIZE: usize = 64 * 2;

#[derive(Serialize, Deserialize)]
pub enum DragState {
    None,
    From(V2i),
}

pub trait TileTextureProvider {
    type Texture;

    fn alloc(&mut self, pixels_rgba: &[u8]) -> Self::Texture;
    fn free(&mut self, texture: Self::Texture);
}

#[derive(Serialize, Deserialize)]
/// After so many updates, i am not entierly sure what this struct is supposed to become
pub struct Fractal<T> {
    // state
    pub pos: Viewport,
    pub iter: i32,
    pub kind: TileType,

    // Input stuff
    pub pause: bool,
    pub debug: bool,
    drag: DragState,

    #[serde(skip)]
    // this uses a workaround to prevent incorrect `T: Default` bounds.
    // see: https://github.com/serde-rs/serde/issues/1541
    #[serde(default = "TileStorage::new")]
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
            drag: DragState::None,
            pause: false,
            debug: false,
            tile_builder: None,
            queue: Arc::new(Mutex::new(WorkQueue::new())),

            iter: 64,
            kind: TileType::Mandelbrot,
        }
    }

    pub fn update_tiles(&mut self, texture_creator: &mut impl TileTextureProvider<Texture = T>) {
        // This recreates tile builders when entire struct is deserialized
        if self.tile_builder.is_none() {
            self.tile_builder = Some(TileBuilder::new(Arc::clone(&self.queue)));
        }

        let mut queue = match self.queue.try_lock() {
            Err(_) => return,
            Ok(q) => q,
        };

        self.tiles
            .update_tiles(&mut queue, self.kind, self.iter, &self.pos, texture_creator);
    }

    pub fn do_input(&mut self, input: &Input, dt: DeltaTime) {
        self.pos.zoom_in_at(0.3 * input.scroll as f64, input.mouse);
        self.pos.translate({
            let mut p = dt.0 as f64 * input.dir_move * 2.0 * self.pos.size_in_pixels.x;
            p.y *= -1.0;
            to_v2i(p)
        });
        self.pos.zoom_in(dt.0 as f64 * input.dir_look.y * 3.5);

        if let DragState::From(p1) = self.drag {
            self.pos.translate(p1 - input.mouse);
        }

        self.drag = if input.mouse_down.is_down {
            DragState::From(input.mouse)
        } else {
            DragState::None
        };

        // TODO: in the future we want some kind of ui, or cli interface
        if input.button(InputAction::F1).went_down() {
            self.pause = !self.pause;
        }

        if input.button(InputAction::F2).went_down() {
            self.debug = !self.debug;
        }

        if input.button(InputAction::F3).went_down() {
            self.iter += 40;
        }

        if input.button(InputAction::F4).went_down() {
            self.iter -= 40;
            self.iter = self.iter.max(3);
        }

        if input.button(InputAction::F7).went_down() {
            self.kind = self.kind.next();
        }
    }
}

impl<T> Default for Fractal<T> {
    fn default() -> Self {
        Fractal::new(Vector2::new(800, 600))
    }
}
