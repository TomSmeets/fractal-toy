use crate::math::*;
use palette::*;

use sdl2::pixels::*;
use sdl2::rect::*;
use sdl2::render::*;

use std::collections::hash_map::{Entry, HashMap};

use std::sync::{Arc, Mutex};
use std::thread;

use crate::input::{Input, InputAction};
use crate::sdl::Sdl;
use crate::viewport::Viewport;
use crate::window::Window;
use crate::*;

pub mod gen;
pub mod tile;

use self::gen::*;
use self::tile::{TileContent, TilePos};

static TEXTURE_SIZE: usize = 64 * 4;

pub enum DragState {
    None,
    From(V2),
}

pub enum TileState {
    Queued,
    Working,
    Done(TileContent),
}

pub struct Fractal {
    pub textures: Arc<Mutex<HashMap<TilePos, TileState>>>,
    pub pos: Viewport,
    pub drag: DragState,
}

impl Fractal {
    pub fn new() -> Self {
        let h: HashMap<TilePos, TileState> = HashMap::new();
        let q = Arc::new(Mutex::new(h));

        for _ in 0..4 {
            let q = q.clone();
            thread::spawn(move || loop {
                let next: Option<TilePos> = {
                    let mut l = q.lock().unwrap();
                    let p = l
                        .iter_mut()
                        .filter(|(_, x)| matches!(x, TileState::Queued))
                        .map(|(p, t)| (*p, t))
                        .min_by_key(|(p, _)| p.z);

                    match p {
                        Some((p, t)) => {
                            *t = TileState::Working;
                            Some(p)
                        },

                        None => None,
                    }
                };

                match next {
                    Some(p) => {
                        let t = TileContent::new(p);
                        let mut map = q.lock().unwrap();
                        map.insert(p, TileState::Done(t));
                    },
                    None => thread::sleep(std::time::Duration::from_millis(50)),
                }
            });
        }

        Fractal {
            textures: q,
            pos: Viewport::new(),
            drag: DragState::None,
        }
    }

    pub fn update(&mut self, time: &Time, sdl: &mut Sdl, window: &Window, input: &Input) {
        let mouse_in_view = screen_to_view(window, input.mouse);
        self.pos.zoom_in(0.1 * input.scroll as f64, mouse_in_view);

        self.pos.translate(time.dt as f64 * input.dir_move * 2.0);
        self.pos
            .zoom_in(time.dt as f64 * input.dir_look.y * 4.0, V2::new(0.5, 0.5));

        if let DragState::From(p1) = self.drag {
            self.pos.translate(p1 - mouse_in_view);
        }

        self.drag = if input.mouse_down.is_down {
            DragState::From(mouse_in_view)
        } else {
            DragState::None
        };

        if input.is_down(InputAction::Y) {
            let t = self.textures.lock();
            t.unwrap().clear();
        }

        {
            let ps = self.pos.get_pos_all();
            let mut t = self.textures.lock().unwrap();
            for p in ps {
                let e = t.entry(p);
                match e {
                    Entry::Occupied(_) => (),
                    Entry::Vacant(e) => {
                        e.insert(TileState::Queued);
                    },
                };
            }
        }

        {
            let t = self.textures.lock().unwrap();
            let mut vs: Vec<_> = t.iter().collect();
            vs.sort_unstable_by_key(|(p, _)| p.z);
            let mut texture = sdl
                .canvas
                .texture_creator()
                .create_texture_streaming(
                    PixelFormatEnum::RGBA8888,
                    TEXTURE_SIZE as u32,
                    TEXTURE_SIZE as u32,
                )
                .unwrap();

            let mut count_empty = 0;
            let mut count_working = 0;
            let mut count_full = 0;

            for (p, v) in &vs {
                let r = self.pos_to_rect(window, p);
                match v {
                    TileState::Done(v) => {
                        v.to_sdl(&mut texture);
                        sdl.canvas.copy(&texture, None, Some(r)).unwrap();
                        count_full += 1;
                    },
                    TileState::Queued => {
                        count_empty += 1;
                    },
                    TileState::Working => {
                        count_working += 1;
                    },
                };
            }

            println!(
                "{}+{} / {}",
                count_working,
                count_empty,
                count_empty + count_full + count_working
            );

            unsafe {
                texture.destroy();
            }
        }

        if true {
            let self_zoom = self.pos.zoom;
            self.textures.lock().unwrap().retain(|p, _| {
                let z_min = self_zoom - 8.0;
                let z_max = self_zoom + 3.0;

                let s = self.pos.scale();
                let p_min = self.pos.offset;
                let p_max = self.pos.offset + Vector2::new(s, s);

                let mut keep = (p.z as f64) > z_min && (p.z as f64) < z_max;
                if keep {
                    let [x, y, z] = p.to_f64();
                    let s0 = z;
                    keep = x < p_max.x + s0
                        && y < p_max.y + s0
                        && x > p_min.x - s0
                        && y > p_min.y - s0;
                }
                keep
            });
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fractal() {
        TileContent::new(TilePos { x: 0, y: 0, z: 0 });
    }
}
