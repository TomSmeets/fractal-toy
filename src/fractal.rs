use crate::math::*;
use crate::time::DeltaTime;
use crate::Input;
use serde::{Deserialize, Serialize};

pub mod builder;
pub mod viewport;

mod content;
pub use self::content::TileContent;

use self::builder::TileBuilder;
use self::builder::TileParams;
use self::builder::TileRequest;
use self::builder::TileType;
use self::viewport::Viewport;
use crate::tilemap::Task;
use crate::tilemap::TileStorage;
use crossbeam_channel::bounded;
use crossbeam_channel::{Receiver, Sender};

// We are blending the textures
pub const PADDING: u32 = 1;
pub const TEXTURE_SIZE: usize = 64 * 2;

/// Something that can build textures from tile pixels
pub trait TileTextureProvider {
    type Texture;
    fn alloc(&mut self, pixels_rgba: &[u8]) -> Self::Texture;
    fn free(&mut self, texture: Self::Texture);
}

pub struct QueueHandler {
    pub tx: Sender<TileRequest>,
    pub rx: Receiver<(TileRequest, TileContent)>,
    pub builder: TileBuilder,
}

/// After so many updates, i am not entierly sure what this struct is supposed to become
#[derive(Serialize, Deserialize)]
pub struct Fractal<T> {
    // state
    pub pos: Viewport,
    dirty: bool,
    pub params: TileParams,

    // this uses a workaround to prevent incorrect `T: Default` bounds.
    // see: https://github.com/serde-rs/serde/issues/1541
    #[serde(skip, default = "TileStorage::new")]
    pub tiles: TileStorage<T>,

    #[serde(skip)]
    pub queue: Option<QueueHandler>,
}

impl<T> Fractal<T> {
    pub fn new(size: Vector2<u32>) -> Self {
        Fractal {
            tiles: TileStorage::new(),
            pos: Viewport::new(size),
            queue: None,
            dirty: false,
            params: TileParams {
                kind: TileType::Mandelbrot,
                iterations: 64,
                padding: 1,
                resolution: TEXTURE_SIZE as u32,
            },
        }
    }

    pub fn update_tiles(&mut self, texture_creator: &mut impl TileTextureProvider<Texture = T>) {
        if self.dirty {
            self.tiles.clear();
            self.dirty = false;
        }

        // This recreates tile builders when entire struct is deserialized
        if self.queue.is_none() {
            // bounds is the amount of tiles that are built within one frame
            // it should be largen enough to saturate the tile builders
            // however, all tiles insed this channel will be built, so making it too big will build
            // tiles that might have left the screen
            // TODO: either predict this boundst
            // TODO: or dynamically change it
            // TODO: or make it small and provide tiles more than once per frame
            let (in_tx, in_rx) = bounded(32);
            let (out_tx, out_rx) = bounded(32);
            let q = QueueHandler {
                tx: in_tx,
                rx: out_rx,
                builder: TileBuilder::new(in_rx, out_tx),
            };
            self.queue = Some(q);
            println!("created queue")
        }

        let queue = self.queue.as_mut().unwrap();
        queue.builder.update();

        // send todo to builders
        for (r, t) in self.tiles.iter_mut() {
            if let Task::Todo = t {
                if let Ok(_) = queue.tx.try_send(TileRequest {
                    pos: *r,
                    params: self.params,
                }) {
                    *t = Task::Doing;
                } else {
                    break;
                }
            }
        }

        // read from builders
        while let Ok((r, t)) = queue.rx.try_recv() {
            if let Some(v) = self.tiles.get_mut(&r.pos) {
                if r.params == self.params {
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
