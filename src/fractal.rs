use crate::math::*;
use crate::time::DeltaTime;
use crate::Input;
use serde::{Deserialize, Serialize};

mod builder;
mod content;
mod queue;
mod viewport;

pub use self::content::TileContent;

use self::builder::TileBuilder;
use self::builder::TileParams;
use self::builder::TileType;
use self::queue::Queue;
use self::viewport::Viewport;
use crate::tilemap::Task;
use crate::tilemap::TileMap;

// We are blending the textures
pub const PADDING: u32 = 1;
pub const TEXTURE_SIZE: usize = 64 * 2;

/// Something that can build textures from tile pixels
pub trait TileTextureProvider {
    type Texture;
    fn alloc(&mut self, pixels_rgba: &[u8]) -> Self::Texture;
    fn free(&mut self, texture: Self::Texture);
}

pub struct Builder {
    pub queue: Queue,
    pub builder: TileBuilder,
}

impl Builder {
    pub fn new() -> Self {
        let queue = Queue::new();
        let builder = TileBuilder::new(queue.handle());
        Builder { queue, builder }
    }
}

/// After so many updates, i am not entierly sure what this struct is supposed to become
#[derive(Serialize, Deserialize)]
pub struct Fractal<T> {
    // state
    pub pos: Viewport,

    params: TileParams,

    // this uses a workaround to prevent incorrect `T: Default` bounds.
    // see: https://github.com/serde-rs/serde/issues/1541
    #[serde(skip, default = "TileMap::new")]
    pub tiles: TileMap<T>,

    #[serde(skip, default = "Builder::new")]
    pub builder: Builder,
}

impl<T> Fractal<T> {
    pub fn new(size: Vector2<u32>) -> Self {
        Fractal {
            tiles: TileMap::new(),
            pos: Viewport::new(size),
            builder: Builder::new(),
            params: TileParams {
                kind: TileType::Mandelbrot,
                iterations: 64,
                padding: 1,
                resolution: TEXTURE_SIZE as u32,
            },
        }
    }

    pub fn update_tiles(&mut self, texture_creator: &mut impl TileTextureProvider<Texture = T>) {
        let queue = &mut self.builder;
        queue.builder.update();

        // send todo to builders
        for (r, t) in self.tiles.iter_mut() {
            if let Task::Todo = t {
                if let Ok(_) = queue.queue.try_send(self.params, *r) {
                    *t = Task::Doing;
                } else {
                    break;
                }
            }
        }

        // read from builders
        while let Ok(r) = queue.queue.try_recv(self.params) {
            if let Some((p, t)) = r {
                if let Some(v) = self.tiles.get_mut(&p) {
                    let t = texture_creator.alloc(&t.pixels);
                    if let Task::Doing = v {
                        *v = Task::Done(t);
                    }
                }
            }
        }

        let new_iter = self.pos.get_pos_all();
        self.tiles
            .update_with(new_iter, |_, v| texture_creator.free(v));
    }

    pub fn do_input(&mut self, input: &Input, dt: DeltaTime) {
        if input.scroll != 0 {
            self.pos.zoom_in_at(0.3 * input.scroll as f64, input.mouse);
        }

        self.pos.translate({
            let mut p = dt.0 as f64 * input.dir_move * 2.0 * self.pos.size_in_pixels().x;
            p.y *= -1.0;
            to_v2i(p)
        });
        self.pos.zoom_in(dt.0 as f64 * input.zoom as f64 * 3.5);
        self.pos.translate(-input.drag);

        // TODO: in the future we want some kind of ui, or cli interface
        if input.iter_inc {
            self.params.iterations += 40;
            self.tiles.clear();
        }

        if input.iter_dec {
            self.params.iterations -= 40;
            self.params.iterations = self.params.iterations.max(3);
            self.tiles.clear();
        }

        if input.cycle {
            self.params.kind = self.params.kind.next();
            self.tiles.clear();
        }
    }
}
