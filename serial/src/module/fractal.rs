use crate::{
    math::*,
    module::{input::InputAction, Input, Sdl, Time, Window},
};
use sdl2::rect::Rect;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, RwLock},
};

pub mod atlas;
pub mod builder;
pub mod tile;
pub mod viewport;

use self::{
    atlas::Atlas,
    builder::{
        queue::{TileQueue, WorkQueue},
        threaded::ThreadedTileBuilder,
        TileRequest, TileType,
    },
    tile::{TileContent, TilePos},
    viewport::Viewport,
};

const TEXTURE_SIZE: usize = 64 * 2;

#[derive(Serialize, Deserialize)]
pub enum DragState {
    None,
    From(V2),
}

struct SortedIter<I> {
    itr: I,
}

enum ComparedValue<L, R> {
    Left(L),
    Right(R),
    Both(L, R),
}

use std::iter::Peekable;

struct CompareIter<I, J, FCmp>
where
    I: Iterator,
    J: Iterator,
    FCmp: FnMut(&I::Item, &J::Item) -> Ordering,
{
    left: Peekable<I>,
    right: Peekable<J>,
    fcmp: FCmp,
}

impl<I, J, FCmp> Iterator for CompareIter<I, J, FCmp>
where
    I: Iterator,
    J: Iterator,
    FCmp: FnMut(&I::Item, &J::Item) -> Ordering,
{
    type Item = ComparedValue<I::Item, J::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let ord = match (self.left.peek(), self.right.peek()) {
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (None, None) => return None,
            (Some(old), Some(new)) => (self.fcmp)(old, new),
        };

        Some(match ord {
            Ordering::Less => ComparedValue::Left(self.left.next().unwrap()),
            Ordering::Equal => {
                ComparedValue::Both(self.left.next().unwrap(), self.right.next().unwrap())
            },
            Ordering::Greater => ComparedValue::Right(self.right.next().unwrap()),
        })
    }
}

fn compare_sorted_iters<I, J, FC, F>(mut old_iter: I, mut new_iter: J, mut cmp: FC, mut f: F)
where
    I: Iterator,
    J: Iterator,
    F: FnMut(ComparedValue<I::Item, J::Item>),
    FC: FnMut(&I::Item, &J::Item) -> Ordering,
{
}

impl<I, K, V> SortedIter<I>
where
    I: Iterator<Item = (K, V)>,
    K: Ord,
{
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
    pub tile_builder: Option<(ThreadedTileBuilder)>,

    #[serde(skip)]
    pub queue: Arc<Mutex<TileQueue>>,

    pub iter: i32,
    pub kind: TileType,
}

impl Fractal {
    pub fn new() -> Self {
        let map: TileMap = TileMap::new();

        Fractal {
            textures: map,
            pos: Viewport::new(),
            drag: DragState::None,
            atlas: Atlas::new(TEXTURE_SIZE as u32),
            pause: false,
            debug: false,
            tile_builder: None,
            queue: Arc::new(Mutex::new(WorkQueue::new())),

            iter: 64,
            kind: TileType::Mandelbrot,
        }
    }

    pub fn zoom(&mut self, amount: f32, position: Option<V2>) {
        let position = position.unwrap_or_else(|| V2::new(0.5, 0.5));
        self.pos.zoom_in(amount as f64, position);
    }

    pub fn translate(&mut self, offset: V2) {
        self.pos.translate(offset);
    }

    pub fn update(&mut self, time: &Time, sdl: &mut Sdl, window: &Window, input: &Input) {
        let mouse_in_view = screen_to_view(window, input.mouse);
        self.zoom(0.3 * input.scroll as f32, Some(mouse_in_view));
        self.translate(time.dt as f64 * input.dir_move * 2.0);
        self.zoom(time.dt * input.dir_look.y as f32 * 3.5, None);

        if self.tile_builder.is_none() {
            self.tile_builder = Some(ThreadedTileBuilder::new(Arc::clone(&self.queue)));
        }

        if let DragState::From(p1) = self.drag {
            self.pos.translate(p1 - mouse_in_view);
        }

        self.drag = if input.mouse_down.is_down {
            DragState::From(mouse_in_view)
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

        if !self.pause || input.button(InputAction::F3).went_down() {
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

                // items that finished computing
                //            let mut done_iter = ...;

                let mut items_to_remove = Vec::new();
                let mut items_to_insert = Vec::new();
                let mut items_to_retain = Vec::new();

                let iter = CompareIter {
                    left: old_iter.peekable(),
                    right: new_iter.peekable(),
                    fcmp: |l, r| l.0.cmp(r),
                };

                for i in iter {
                    match i {
                        ComparedValue::Left(l) => items_to_remove.push(l), // only in old_iter
                        ComparedValue::Right(r) => items_to_insert.push(r), // only in new_iter
                        ComparedValue::Both(l, _) => items_to_retain.push(l), // old and new
                    };
                }

                // iterate over all visible position and queue those tiles
                let mut todo = items_to_insert;
                todo.reverse();
                println!("--- queue ---");
                println!("todo_old: {:?}", q.todo.len());
                println!("todo_new: {:?}", todo.len());
                println!("doing: {:?}", q.doing.len());
                println!("done: {:?}", q.done.len());

                q.todo = todo;

                let mut new = TileMap::new();
                for (p, t) in items_to_retain.into_iter() {
                    new.insert(p, t);
                }
                for (k, mut v) in q.done.drain(..) {
                    if v.region.is_none() {
                        let atlas_region = self.atlas.alloc(sdl);
                        self.atlas.update(&atlas_region, &v.pixels);
                        v.region = Some(atlas_region);
                    }
                    new.insert(k, v);
                }

                let t2 = std::mem::replace(&mut self.textures, new);
                println!("removed {}", t2.len());
                for (_, t) in items_to_remove.into_iter() {
                    if let Some(r) = t.region {
                        self.atlas.remove(r);
                    }
                }
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
            println!("count: {}", self.textures.len());
        }

        // fast stuff
        for (p, v) in self.textures.iter() {
            let r = self.pos_to_rect(window, &p.pos);

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

        if input.button(InputAction::F1).went_down() {
            println!("---- INFO ----");
            self.info(input, window);
        }
    }

    fn pos_to_rect(&self, window: &Window, p: &TilePos) -> Rect {
        let [x, y, z] = p.to_f64();
        let p = V2::new(x as f64, y as f64);
        let w = p + V2::new(z as f64, z as f64);
        let p = self.pos.world_to_view(p);
        let p = view_to_screen(window, p);
        let w = self.pos.world_to_view(w);
        let w = view_to_screen(window, w);
        mk_rect(p, w)
    }

    pub fn info(&self, input: &Input, window: &Window) {
        let mouse_view = screen_to_view(window, input.mouse);
        let mouse_world = self.pos.view_to_world(mouse_view);
        let mouse_view = self.pos.world_to_view(mouse_world);
        let mouse_screen = view_to_screen(window, mouse_view);
        println!("screen  {:6.2} {:6.2}", input.mouse.x, input.mouse.y);
        println!("view    {:6.2} {:6.2}", mouse_view.x, mouse_view.y);
        println!("world   {:6.2} {:6.2}", mouse_world.x, mouse_world.y);
        println!("screen2 {:6.2} {:6.2}", mouse_screen.x, mouse_screen.y);
    }
}

fn screen_to_view(window: &Window, p: V2i) -> V2 {
    let s = window.size.x.max(window.size.y) as f64;
    V2::new(p.x as f64 / s, 1.0 - p.y as f64 / s)
}

fn view_to_screen(window: &Window, p: V2) -> V2i {
    let s = window.size.x.max(window.size.y) as f64;
    V2i::new((p.x * s) as i32, ((1.0 - p.y) * s) as i32)
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
