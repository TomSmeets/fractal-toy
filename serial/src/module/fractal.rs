use crate::{
    math::*,
    module::{input::InputAction, Input, Sdl, Time, Window},
};
use sdl2::rect::Rect;
use serde::{Deserialize, Serialize};
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
        ocl::OCLTileBuilder,
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

type TileMap = BTreeMap<TileRequest, TileContent>;

// queue: [TilePos]
// done:  [Pos, Content]
// TODO: Queried tiles shold be exactly those displayed. All tiles that are not
// directly Queried should be removed. what datastructure is best for this?
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

    #[serde(skip)]
    pub tile_builder: Option<(ThreadedTileBuilder, OCLTileBuilder)>,

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

    pub fn update(&mut self, time: &Time, sdl: &mut Sdl, window: &Window, input: &Input) {
        let mouse_in_view = screen_to_view(window, input.mouse);
        self.pos.zoom_in(0.3 * input.scroll as f64, mouse_in_view);

        self.pos.translate(time.dt as f64 * input.dir_move * 2.0);
        self.pos
            .zoom_in(time.dt as f64 * input.dir_look.y * 3.5, V2::new(0.5, 0.5));

        if self.tile_builder.is_none() {
            self.tile_builder = Some((
                ThreadedTileBuilder::new(Arc::clone(&self.queue)),
                OCLTileBuilder::new(Arc::clone(&self.queue)),
            ));
        }

        if let DragState::From(p1) = self.drag {
            self.pos.translate(p1 - mouse_in_view);
        }

        self.drag = if input.mouse_down.is_down {
            DragState::From(mouse_in_view)
        } else {
            DragState::None
        };

        // TODO: int the future we want some kind of ui, or cli interface
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
            if let Ok(mut q) = self.queue.try_lock() {
                // iterate over all visible position and queue those tiles
                let mut todo = Vec::with_capacity(256);
                let mut new = TileMap::new();
                for p in self.pos.get_pos_all() {
                    let rq = TileRequest {
                        pos: p,
                        iterations: self.iter,
                        kind: self.kind,
                    };
                    let e = self.textures.remove(&rq);

                    match e {
                        Some(e) => {
                            new.insert(rq, e);
                        },
                        None => {
                            if todo.len() < todo.capacity()
                                && !q.doing.iter().any(|x| x == &rq)
                                && !q.done.iter().any(|(y, _)| y == &rq)
                            {
                                todo.push(rq);
                            }
                        },
                    };
                }

                todo.reverse();

                println!("--- queue ---");
                println!("todo_old: {:?}", q.todo.len());
                println!("todo_new: {:?}", todo.len());
                println!("doing: {:?}", q.doing.len());
                println!("done: {:?}", q.done.len());

                q.todo = todo;

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
                for (_, t) in t2 {
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
                // TODO: make rendering seperate from sdl
                sdl.canvas
                    .copy(
                        &self.atlas.texture[atlas_region.index.z as usize],
                        Some(atlas_region.rect_padded().into_sdl()),
                        Some(r),
                    )
                    .unwrap();
            } else {
                panic!("withot region!?");
            }
        }

        if self.debug {
            // Show atlas
            // TODO: show in ui window
            let w = window.size.x / self.atlas.texture.len().max(4) as u32;
            for (i, t) in self.atlas.texture.iter().enumerate() {
                sdl.canvas
                    .copy(t, None, Some(Rect::new(i as i32 * w as i32, 0, w, w)))
                    .unwrap();
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
    V2::new(
        p.x as f64 / s,
        1.0 - p.y as f64 / s,
    )
}

fn view_to_screen(window: &Window, p: V2) -> V2i {
    let s = window.size.x.max(window.size.y) as f64;
    V2i::new(
        (p.x * s) as i32,
        ((1.0 - p.y) * s) as i32,
    )
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
