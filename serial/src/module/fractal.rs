use crate::math::*;
use crate::module::{input::InputAction, Input, Sdl, Time, Window};
use sdl2::rect::Rect;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::{Entry, HashMap};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

pub mod atlas;
pub mod gen;
pub mod tile;
pub mod viewport;

use self::atlas::Atlas;
use self::gen::Gen;
use self::tile::{TileContent, TilePos};
use self::viewport::Viewport;

const TEXTURE_SIZE: usize = 64 * 2;

#[derive(Serialize, Deserialize)]
pub enum DragState {
    None,
    From(V2),
}

type TileMap = Arc<RwLock<HashMap<TilePos, TileContent>>>;

pub struct Worker {
    quit: Arc<RwLock<bool>>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(gen: &mut Arc<RwLock<Gen>>, map: &mut TileMap) -> Worker {
        let quit = Arc::new(RwLock::new(false));

        let handle = {
            let map = Arc::clone(&map);
            let gen = Arc::clone(&gen);
            let quit = Arc::clone(&quit);
            thread::spawn(move || worker(gen, map, quit))
        };

        Worker {
            quit,
            handle: Some(handle),
        }
    }

    pub fn quit(&mut self) {
        *(self.quit.write().unwrap()) = true;
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.quit();
        self.handle.take().unwrap().join().unwrap();
    }
}

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
}

pub fn worker(gen: Arc<RwLock<Gen>>, q: TileMap, quit: Arc<RwLock<bool>>) {
    loop {
        if *quit.read().unwrap() {
            break;
        }

        let next: Option<TilePos> = {
            let mut l = q.write().unwrap();
            let p = l
                .iter_mut()
                .filter(|(_, x)| x.dirty && !x.working)
                .map(|(p, t)| (*p, t))
                .min_by_key(|(p, _)| p.z);

            match p {
                Some((p, t)) => {
                    t.working = true;
                    Some(p)
                },

                None => None,
            }
        };

        match next {
            Some(p) => {
                let g = gen.read().unwrap();
                let mut t = TileContent::new();
                t.generate(&g, p);
                let mut map = q.write().unwrap();
                match map.entry(p) {
                    Entry::Occupied(mut e) => {
                        // we are going to drop the previous tile, so make sure it does not contain
                        // a atlas region that would be a bug for now
                        // in the future wi might just update the pixels and not drop the tile!
                        // assert!(e.get().region.is_none());
                        e.insert(t);
                    },
                    Entry::Vacant(_) => {
                        // a tile got removed while we where working on it
                        // let's just ignore it for now
                    },
                };
            },
            None => {
                thread::yield_now();
                thread::sleep(std::time::Duration::from_millis(50));
            },
        }
    }
}

impl Fractal {
    pub fn new() -> Self {
        let map: TileMap = Arc::new(RwLock::new(HashMap::new()));
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
        }
    }

    fn spaw_workers(&mut self) {
        let n = (sdl2::cpuinfo::cpu_count() - 1).max(1);
        println!("spawning {} workers", n);
        for _ in 0..n {
            self.workers
                .push(Worker::new(&mut self.gen, &mut self.textures));
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
        let t = self.textures.try_write();
        match t {
            Ok(mut t) => {
                if !self.pause {
                    // Mark all entires as potentialy old
                    for (_, e) in t.iter_mut() {
                        e.old = true;
                    }

                    // iterate over all visible position and queue those tiles
                    for p in ps {
                        let e = t.entry(p);
                        match e {
                            Entry::Occupied(mut e) => {
                                e.get_mut().old = false;
                            },
                            Entry::Vacant(e) => {
                                e.insert(TileContent::new());
                            },
                        };
                    }

                    if input.is_down(InputAction::Y) {
                        for (_, t) in t.iter_mut() {
                            t.old = true;
                            let r = t.region.take();
                            if let Some(r) = r {
                                self.atlas.remove(r);
                            }
                        }
                        self.atlas = Atlas::new(self.atlas.res);
                    }

                    for (_, t) in t.iter_mut() {
                        if t.old {
                            let r = t.region.take();
                            if let Some(r) = r {
                                self.atlas.remove(r);
                            }
                        } else {
                            if !t.dirty && !t.working && !t.pixels.is_empty() && t.region.is_none()
                            {
                                let atlas_region = self.atlas.alloc(sdl);
                                self.atlas.update(&atlas_region, &t.pixels);
                                t.region = Some(atlas_region);
                            }
                        }
                    }

                    // remove tiles that we did not encounter
                    t.retain(|_, t| if t.old { false } else { true });
                }
            },
            Err(_) => (),
        }

        let t = self.textures.read();
        match t {
            Ok(t) => {
                let mut vs: Vec<_> = t.iter().collect();
                vs.sort_unstable_by_key(|(p, _)| p.z);

                let mut count_empty = 0;
                let mut count_working = 0;
                let mut count_full = 0;

                for (p, v) in vs.iter() {
                    let r = self.pos_to_rect(window, p);

                    if v.working {
                        if self.debug {
                            sdl.canvas.draw_rect(r).unwrap();
                        }
                        count_working += 1;
                    } else if v.dirty {
                        count_empty += 1;
                    } else {
                        if let Some(atlas_region) = &v.region {
                            // TODO: make rendering seperate from
                            sdl.canvas
                                .copy(
                                    &self.atlas.texture[atlas_region.index.z as usize],
                                    Some(atlas_region.rect().into_sdl()),
                                    Some(r),
                                )
                                .unwrap();
                        }
                        count_full += 1;
                    }
                }

                if self.debug {
                    let w = window.size.x / self.atlas.texture.len().max(4) as u32;
                    for (i, t) in self.atlas.texture.iter().enumerate() {
                        sdl.canvas
                            .copy(t, None, Some(Rect::new(i as i32 * w as i32, 0, w, w)))
                            .unwrap();
                    }

                    println!(
                        "{}+{} / {}",
                        count_working,
                        count_empty,
                        count_empty + count_full + count_working
                    );
                }
            },
            Err(_) => {},
        }

        if input.is_down(InputAction::F1) {
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
