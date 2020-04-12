use crate::iter::compare::{CompareIter, ComparedValue};
use crate::math::*;
use crate::module::{input::InputAction, Input, Time, Window};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

pub mod atlas;
pub mod builder;
pub mod tile;
pub mod viewport;

use self::atlas::TileTextureProvider;
use self::builder::queue::{TileQueue, WorkQueue};
use self::builder::TileBuilder;
use self::builder::{TileRequest, TileType};
use self::tile::TilePos;
use self::viewport::Viewport;

pub const TEXTURE_SIZE: usize = 64 * 2;

#[derive(Serialize, Deserialize)]
pub enum DragState {
    None,
    From(V2i),
}

// pos -> pixels | atlas
// queue: [TilePos]
// done:  [Pos, Content]
// TODO: Queried tiles should be exactly those displayed. All tiles that are not
// directly Queried should be removed. what data structure is best for this?
// multiple gen types, like threaded gen, etc
#[derive(Serialize, Deserialize)]
pub struct Fractal<T> {
    pos: Viewport,

    // this uses a workaround to prevent incorrect `T: Default` bounds.
    // see: https://github.com/serde-rs/serde/issues/1541
    #[serde(skip)]
    #[serde(default = "Default::default")]
    textures: Vec<(TileRequest, T)>,

    #[serde(skip)]
    queue: Arc<Mutex<TileQueue>>,

    #[serde(skip)]
    tile_builder: Option<TileBuilder>,

    pub iter: i32,
    pub kind: TileType,

    pub pause: bool,
    pub debug: bool,
    drag: DragState,
    frame_counter: u32,

    #[serde(skip)]
    items_to_insert: Vec<TileRequest>,

    #[serde(skip)]
    #[serde(default = "Default::default")]
    items_to_retain: Vec<(TileRequest, T)>,
}

impl<T> Fractal<T> {
    pub fn new() -> Self {
        Fractal {
            textures: Vec::new(),
            pos: Viewport::new(Vector2::new(800, 600)),
            drag: DragState::None,
            pause: false,
            debug: false,
            tile_builder: None,
            queue: Arc::new(Mutex::new(WorkQueue::new())),

            iter: 64,
            kind: TileType::Mandelbrot,
            frame_counter: 0,

            items_to_insert: Vec::new(),
            items_to_retain: Vec::new(),
        }
    }

    pub fn update_tiles(&mut self, texture_creator: &mut impl TileTextureProvider<Texture = T>) {
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
        let old_iter = self.textures.drain(..);
        // items we should render this frame
        let iter = self.iter;
        let kind = self.kind;
        let new_iter = self.pos.get_pos_all().map(|pos| TileRequest {
            pos,
            iterations: iter,
            kind,
        });

        assert!(self.items_to_insert.is_empty());
        assert!(self.items_to_retain.is_empty());

        let iter = CompareIter::new(old_iter, new_iter, |l, r| l.0.cmp(r));

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
                        self.items_to_insert.push(r)
                    }
                },
                ComparedValue::Both(l, _) => {
                    // this value should be retained, as it is in new_iter and old_iter
                    self.items_to_retain.push(l)
                },
            }
        }

        println!("--- queue ---");
        println!("retain:   {:?}", self.items_to_retain.len());
        println!("todo:     {:?}", self.items_to_insert.len());
        println!("todo_old: {:?}", q.todo.len());
        println!("doing:    {:?}", q.doing.len());
        println!("done:     {:?}", q.done.len());

        // swap new and old todos
        self.items_to_insert.reverse();
        std::mem::swap(&mut q.todo, &mut self.items_to_insert);
        self.items_to_insert.clear();

        // TODO: add sorted done at beginning when iterating
        // q.done.sort_unstable_by(|(r1, _), (r2, _)| r1.cmp(r2));
        for (k, v) in q.done.drain(..) {
            let atlas_region = texture_creator.alloc(&v.pixels);
            // TODO: what is faster sort or iter?
            self.items_to_retain.push((k, atlas_region));
        }

        // This should use timsort and should be pretty fast for this usecase
        // Note that in this spesific case, the normal sort will probably be faster than
        // the unstable sort TODO: profile :)
        self.items_to_retain.sort_by(|(r1, _), (r2, _)| r1.cmp(r2));
        std::mem::swap(&mut self.items_to_retain, &mut self.textures);
    }

    pub fn update(
        &mut self,
        texture_provider: &mut impl TileTextureProvider<Texture = T>,
        time: &Time,
        window: &Window,
        input: &Input,
    ) {
        self.frame_counter += 1;

        self.pos.resize(window.size);

        self.pos.zoom_in_at(0.3 * input.scroll as f64, input.mouse);
        self.pos.translate({
            let mut p = time.dt as f64 * input.dir_move * 2.0 * self.pos.size_in_pixels.x;
            p.y *= -1.0;
            to_v2i(p)
        });
        self.pos.zoom_in(time.dt as f64 * input.dir_look.y * 3.5);

        if self.tile_builder.is_none() {
            self.tile_builder = Some(TileBuilder::new(Arc::clone(&self.queue)));
        }

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

        if !self.pause {
            self.update_tiles(texture_provider);
        }

        // draw stuff
        for (p, atlas_region) in self.textures.iter() {
            let r = self.pos_to_rect(&p.pos);
            // TODO: make rendering separate from sdl
            texture_provider.draw(atlas_region, r);
        }
    }

    fn pos_to_rect(&self, p: &TilePos) -> Rect {
        let [x, y, z] = p.to_f64();
        let min = V2::new(x, y);
        let max = min + V2::new(z, z);
        let min = self.pos.world_to_screen(min);
        let max = self.pos.world_to_screen(max);
        mk_rect(min, max)
    }
}

fn mk_rect(a: V2i, b: V2i) -> Rect {
    let min_x = a.x.min(b.x);
    let min_y = a.y.min(b.y);

    let max_x = a.x.max(b.x);
    let max_y = a.y.max(b.y);

    let width = max_x.saturating_sub(min_x);
    let height = max_y.saturating_sub(min_y);

    Rect {
        pos: V2i::new(min_x, min_y),
        size: V2i::new(width, height),
    }
}

impl<T> Default for Fractal<T> {
    fn default() -> Self {
        Fractal::new()
    }
}
