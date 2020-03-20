use crate::math::*;
use crate::module::{input::InputAction, Input, Sdl, Time, Window};
use sdl2::rect::Rect;
use serde::{Deserialize, Serialize};
// use std::collections::btree_map::{Entry, BTreeMap};
use std::collections::hash_map::{Entry, HashMap};
use std::sync::{Arc, Mutex, RwLock};

pub mod atlas;
pub mod gen;
pub mod tile;
pub mod viewport;
pub mod worker;

use self::atlas::Atlas;
use self::gen::Gen;
use self::tile::{TileContent, TilePos};
use self::viewport::Viewport;
use self::worker::*;

const TEXTURE_SIZE: usize = 64 * 2;

#[derive(Serialize, Deserialize)]
pub enum DragState {
    None,
    From(V2),
}

type TileMap = HashMap<TilePos, TileContent>;

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
    pub gen: Arc<RwLock<Gen>>,
    pub atlas: Atlas,
    pub pause: bool,
    pub debug: bool,

    #[serde(skip)]
    pub workers: Vec<Worker>,

    #[serde(skip)]
    pub queue: Arc<Mutex<TileQueue>>,
}

impl Fractal {
    pub fn new() -> Self {
        let map: TileMap = HashMap::new();
        let gen = Arc::new(RwLock::new(Gen::new()));

        Fractal {
            textures: map,
            pos: Viewport::new(),
            drag: DragState::None,
            gen,
            atlas: Atlas::new(TEXTURE_SIZE as u32),
            pause: false,
            debug: false,
            workers: Vec::new(),
            queue: Arc::new(Mutex::new(WorkQueue::new())),
        }
    }

    fn spaw_workers(&mut self) {
        let n = (sdl2::cpuinfo::cpu_count() - 1).max(1);
        println!("spawning {} workers", n);
        for _ in 0..8 {
            self.workers
                .push(Worker::new(&mut self.gen, &mut self.queue));
        }
    }

    pub fn update(&mut self, time: &Time, sdl: &mut Sdl, window: &Window, input: &Input) {
        let mouse_in_view = screen_to_view(window, input.mouse);
        self.pos.zoom_in(0.1 * input.scroll as f64, mouse_in_view);

        self.pos.translate(time.dt as f64 * input.dir_move * 2.0);
        self.pos
            .zoom_in(time.dt as f64 * input.dir_look.y * 4.0, V2::new(0.5, 0.5));

        if self.workers.is_empty() {
            self.spaw_workers();
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

        let ps = self.pos.get_pos_all();
        if !self.pause || input.button(InputAction::F3).went_down() {
            if let Ok(mut q) = self.queue.try_lock() {
                // Mark all entires as potentialy old
                for (_, e) in self.textures.iter_mut() {
                    e.old = true;
                }

                // iterate over all visible position and queue those tiles
                let mut todo = Vec::with_capacity(16);
                for p in ps {
                    let e = self.textures.get_mut(&p);
                    match e {
                        Some(mut e) => {
                            e.old = false;
                        },
                        None => {
                            if todo.len() < todo.capacity()
                                && !q.doing.iter().any(|x| *x == p)
                                && !q.done.iter().any(|(y, _)| *y == p)
                            {
                                todo.push(p);
                            }
                        },
                    };
                }
                todo.sort_by_key(|p| -p.z);
                q.todo = todo;

                println!("--- queue ---");
                println!("todo: {:?}", q.todo.len());
                println!("doing: {:?}", q.doing.len());
                println!("done: {:?}", q.done.len());

                for (k, v) in q.done.drain(..) {
                    self.textures.insert(k, v);
                }
            }

            if input.button(InputAction::Y).went_down() {
                for (_, t) in self.textures.iter_mut() {
                    t.old = true;
                    if let Some(r) = t.region.take() {
                        self.atlas.remove(r);
                    }
                }
                self.atlas = Atlas::new(self.atlas.res);
            }

            for (_, t) in self.textures.iter_mut() {
                if t.old {
                    let r = t.region.take();
                    if let Some(r) = r {
                        self.atlas.remove(r);
                    }
                } else {
                    if !t.dirty && !t.working && !t.pixels.is_empty() && t.region.is_none() {
                        let atlas_region = self.atlas.alloc(sdl);
                        self.atlas.update(&atlas_region, &t.pixels);
                        t.region = Some(atlas_region);
                    }
                }
            }

            // remove tiles that we did not encounter
            self.textures.retain(|_, t| !t.old);
        }

        // fast stuff
        {
            let mut vs: Vec<_> = self.textures.iter().collect();
            vs.sort_unstable_by_key(|(p, _)| p.z);

            for (p, v) in vs.iter() {
                let r = self.pos_to_rect(window, p);

                if let Some(atlas_region) = &v.region {
                    // TODO: make rendering seperate from sdl
                    sdl.canvas
                        .copy(
                            &self.atlas.texture[atlas_region.index.z as usize],
                            Some(atlas_region.rect().into_sdl()),
                            Some(r),
                        )
                        .unwrap();
                }
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

impl Drop for Fractal {
    fn drop(&mut self) {
        for w in self.workers.iter_mut() {
            w.quit();
        }
    }
}

fn screen_to_view(window: &Window, p: V2i) -> V2 {
    V2::new(
        p.x as f64 / window.size.x as f64,
        1.0 - p.y as f64 / window.size.y as f64,
    )
}

fn view_to_screen(window: &Window, p: V2) -> V2i {
    V2i::new(
        (p.x * window.size.x as f64) as i32,
        ((1.0 - p.y) * window.size.y as f64) as i32,
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
