use crate::iter::compare::{CompareIter, ComparedValue};
use crate::math::*;
use crate::module::time::DeltaTime;
use crate::module::{input::InputAction, Input};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

pub mod builder;
pub mod tile;
pub mod viewport;

use self::builder::queue::{TileQueue, WorkQueue};
use self::builder::TileBuilder;
use self::builder::{TileRequest, TileType};
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
pub struct Fractal<T> {
    // state
    pub pos: Viewport,
    pub iter: i32,
    pub kind: TileType,

    // Input stuff
    pub pause: bool,
    pub debug: bool,
    drag: DragState,

    // this uses a workaround to prevent incorrect `T: Default` bounds.
    // see: https://github.com/serde-rs/serde/issues/1541
    #[serde(skip)]
    #[serde(default = "Default::default")]
    pub tiles: Vec<(TileRequest, T)>,

    // this temporary storage for when updating tiles
    // stored to prevent reallocations
    #[serde(skip)]
    #[serde(default = "Default::default")]
    next_frame_tiles: Vec<(TileRequest, T)>,

    #[serde(skip)]
    pub queue: Arc<Mutex<TileQueue>>,
    #[serde(skip)]
    tile_builder: Option<TileBuilder>,
}

impl<T> Fractal<T> {
    pub fn new(size: Vector2<u32>) -> Self {
        Fractal {
            tiles: Vec::new(),
            pos: Viewport::new(size),
            drag: DragState::None,
            pause: false,
            debug: false,
            tile_builder: None,
            queue: Arc::new(Mutex::new(WorkQueue::new())),

            iter: 64,
            kind: TileType::Mandelbrot,

            next_frame_tiles: Vec::new(),
        }
    }

    pub fn update_tiles(&mut self, texture_creator: &mut impl TileTextureProvider<Texture = T>) {
        // This recreates tile builders when entire struct is deserialized
        if self.tile_builder.is_none() {
            self.tile_builder = Some(TileBuilder::new(Arc::clone(&self.queue)));
        }

        let mut q = match self.queue.try_lock() {
            Err(_) => return,
            Ok(q) => q,
        };

        // If we have two ordered lists of tile points
        // We can iterate over both lists at the same time and produce three kinds.
        //   drop:    elem(old) && !elem(new)
        //   retain:  elem(old) &&  elem(new)
        //   insert: !elem(old) &&  elem(new)
        //
        // to produce these lists we can do:
        // if old.is_none => insert, new.next();
        // if new.is_none => drop,   old.next();
        // if new.is_none && old.is_none => break;
        // if old < new  => remove, old.next()
        // if old == new => retain, old.next(), new.next()
        // if old > new  => insert, new.next(),
        //
        // oooooo
        // oo......
        // oo......
        // oo......
        //   ......
        //
        // xxxxxx
        // xx....nn
        // xx....nn
        // xx....nn
        //   nnnnnn

        // items we rendered last frame
        let old_iter = self.tiles.drain(..);
        // items we should render this frame
        let iter = self.iter;
        let kind = self.kind;
        let new_iter = self.pos.get_pos_all().map(|pos| TileRequest {
            pos,
            iterations: iter,
            kind,
        });

        assert!(self.next_frame_tiles.is_empty());

        let iter = CompareIter::new(old_iter, new_iter, |l, r| l.0.cmp(r));

        q.todo.clear();
        for i in iter {
            match i {
                ComparedValue::Left((_, t)) => {
                    // only in old_iter, remove value
                    texture_creator.free(t);
                },
                ComparedValue::Right(r) => {
                    // Only in new_iter: enqueue value
                    // TODO: subtract sorted iters instead of this if
                    if !q.doing.contains(&r) && !q.done.iter().any(|x| x.0 == r) {
                        q.todo.push(r)
                    }
                },
                ComparedValue::Both(l, _) => {
                    // this value should be retained, as it is in new_iter and old_iter
                    self.next_frame_tiles.push(l)
                },
            }
        }
        q.todo.reverse();

        // TODO: add sorted done at beginning when iterating
        // q.done.sort_unstable_by(|(r1, _), (r2, _)| r1.cmp(r2));
        for (k, v) in q.done.drain(..) {
            let tile = texture_creator.alloc(&v.pixels);
            // TODO: what is faster sort or iter?
            self.next_frame_tiles.push((k, tile));
        }

        // This should use timsort and should be pretty fast for this usecase
        // Note that in this spesific case, the normal sort will probably be faster than
        // the unstable sort TODO: profile :)
        self.next_frame_tiles.sort_by(|(r1, _), (r2, _)| r1.cmp(r2));
        std::mem::swap(&mut self.next_frame_tiles, &mut self.tiles);
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
            self.kind = match self.kind {
                TileType::Empty => TileType::Mandelbrot,
                TileType::Mandelbrot => TileType::BurningShip,
                TileType::BurningShip => TileType::ShipHybrid,
                TileType::ShipHybrid => TileType::Empty,
            }
        }
    }
}

impl<T> Default for Fractal<T> {
    fn default() -> Self {
        Fractal::new(Vector2::new(800, 600))
    }
}
