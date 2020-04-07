use crate::iter::compare::{CompareIter, ComparedValue};
use crate::math::*;
use crate::module::{input::InputAction, Input, Sdl, Time, Window};
use sdl2::rect::Rect;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

pub mod atlas;
pub mod builder;
pub mod tile;
pub mod viewport;

use self::atlas::Atlas;
use self::builder::queue::{TileQueue, WorkQueue};
use self::builder::threaded::ThreadedTileBuilder;
use self::builder::{TileRequest, TileType};
use self::tile::{TileContent, TilePos};
use self::viewport::Viewport;

const TEXTURE_SIZE: usize = 64 * 2;

#[derive(Serialize, Deserialize)]
pub enum DragState {
    None,
    From(Vector2<i32>),
}

// pos -> pixels | atlas
type TileMap = BTreeMap<TileRequest, TileContent>;

// queue: [TilePos]
// done:  [Pos, Content]
// TODO: Queried tiles should be exactly those displayed. All tiles that are not
// directly Queried should be removed. what data structure is best for this?
// multiple gen types, like threaded gen, etc
#[derive(Serialize, Deserialize)]
pub struct Fractal {
    #[serde(skip)]
    pub textures: TileMap,
    pub pos: Viewport,
    pub drag: DragState,
    pub atlas: Atlas,
    pub pause: bool,
    pub debug: bool,

    // TODO: move out into generic `tile builder` containing all implementations
    #[serde(skip)]
    pub tile_builder: Option<ThreadedTileBuilder>,

    #[serde(skip)]
    pub queue: Arc<Mutex<TileQueue>>,

    pub iter: i32,
    pub kind: TileType,

    pub frame_counter: u32,
}

impl Fractal {
    pub fn new() -> Self {
        let map: TileMap = TileMap::new();

        Fractal {
            textures: map,
            pos: Viewport::new(Vector2::new(800, 600)),
            drag: DragState::None,
            atlas: Atlas::new(TEXTURE_SIZE as u32),
            pause: false,
            debug: false,
            tile_builder: None,
            queue: Arc::new(Mutex::new(WorkQueue::new())),

            iter: 64,
            kind: TileType::Mandelbrot,
            frame_counter: 0,
        }
    }

    pub fn update(&mut self, time: &Time, sdl: &mut Sdl, window: &Window, input: &Input) {
        self.frame_counter += 1;

        let show_info = self.frame_counter % 60 == 0;

        self.pos.resize(window.size);

        self.pos.zoom_in_at(0.3 * input.scroll as f64, input.mouse);
        self.pos.translate({
            let mut p = time.dt as f64 * input.dir_move * 2.0 * self.pos.size_in_pixels.x;
            p.y *= -1.0;
            to_v2i(p)
        });
        self.pos.zoom_in(time.dt as f64 * input.dir_look.y * 3.5);

        if self.tile_builder.is_none() {
            self.tile_builder = Some(ThreadedTileBuilder::new(Arc::clone(&self.queue)));
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
            self.iter += 10;
        }

        if input.button(InputAction::F4).went_down() {
            self.iter -= 10;
            self.iter = self.iter.max(0);
        }

        if input.button(InputAction::F7).went_down() {
            self.kind = match self.kind {
                TileType::Empty => TileType::Mandelbrot,
                TileType::Mandelbrot => TileType::BurningShip,
                TileType::BurningShip => TileType::ShipHybrid,
                TileType::ShipHybrid => TileType::Empty,
            }
        }

        // Tis doesn not have to happen every frame
        if !self.pause && (self.frame_counter % 10 == 0) {
            // it will be faster if we just iterate over two sorted lists.
            // drop existing while existing != new
            // if equal
            if let Ok(mut q) = self.queue.try_lock() {
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

                let t2 = std::mem::replace(&mut self.textures, TileMap::new());
                // items we rendered last frame
                let old_iter = t2.into_iter();
                // items we should render this frame
                let new_iter = self.pos.get_pos_all().map(|pos| TileRequest {
                    pos,
                    iterations: self.iter,
                    kind: self.kind,
                });

                let mut items_to_remove = Vec::new();
                let mut items_to_insert = Vec::new();
                let mut items_to_retain = Vec::new();

                let iter = CompareIter::new(old_iter, new_iter, |l, r| l.0.cmp(r));

                for i in iter {
                    match i {
                        ComparedValue::Left(l) => items_to_remove.push(l), // only in old_iter
                        ComparedValue::Right(r) => if !q.doing.contains(&r) && !q.done.iter().any(|x| x.0 == r) { items_to_insert.push(r) }, // only in new_iter
                        ComparedValue::Both(l, _) => items_to_retain.push(l), // old and new
                    };
                }

                // iterate over all visible position and queue those tiles
                //
                if show_info {
                    println!("--- queue ---");
                    println!("remove:   {:?}", items_to_remove.len());
                    println!("retain:   {:?}", items_to_retain.len());
                    println!("todo:     {:?}", items_to_insert.len());
                    println!("todo_old: {:?}", q.todo.len());
                    println!("doing:    {:?}", q.doing.len());
                    println!("done:     {:?}", q.done.len());
                }

                let mut todo = items_to_insert;
                todo.reverse();
                q.todo = todo;

                let mut new = TileMap::new();
                for (p, t) in items_to_retain.into_iter() {
                    new.insert(p, t);
                }

                for (k, mut v) in q.done.drain(..) {
                    assert!(v.region.is_none());

                    let atlas_region = self.atlas.alloc(sdl);
                    self.atlas.update(&atlas_region, &v.pixels);
                    v.region = Some(atlas_region);

                    // for some reason this happens
                    // TODO: fix?
                    let result = new.insert(k, v);
                    assert!(result.is_none());
                }

                // println!("removed {}", items_to_remove.len());
                for (_, t) in items_to_remove.into_iter() {
                    if let Some(r) = t.region {
                        self.atlas.remove(r);
                    }
                }

                assert!(self.textures.is_empty());
                self.textures = new;
            }

            // if input.button(InputAction::Y).went_down() {
            //     for (_, t) in self.textures.iter_mut() {
            //         t.old = true;
            //         if let Some(r) = t.region.take() {
            //             self.atlas.remove(r);
            //         }
            //     }
            //     self.atlas = Atlas::new(self.atlas.res);
            // }

            // remove tiles that we did not encounter
            // self.textures.retain(|_, t| !t.old);
            // println!("count: {}", self.textures.len());
        }

        // fast stuff
        for (p, v) in self.textures.iter() {
            let r = self.pos_to_rect(&p.pos);

            if let Some(atlas_region) = &v.region {
                // TODO: make rendering separate from sdl
                sdl.canvas_copy(
                    &self.atlas.texture[atlas_region.index.z as usize],
                    Some(atlas_region.rect_padded().into_sdl()),
                    Some(r),
                );
            } else {
                panic!("without region!?");
            }
        }

        if self.debug {
            // Show atlas
            // TODO: show in ui window
            let w = window.size.x / self.atlas.texture.len().max(4) as u32;
            for (i, t) in self.atlas.texture.iter().enumerate() {
                sdl.canvas_copy(t, None, Some(Rect::new(i as i32 * w as i32, 0, w, w)));
            }
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

    let width = max_x - min_x;
    let height = max_y - min_y;

    Rect::new(min_x, min_y, width as u32, height as u32)
}

impl Default for Fractal {
    fn default() -> Fractal {
        Fractal::new()
    }
}
