use crate::math::*;
use serde::{Deserialize, Serialize};

mod builder;
mod content;
mod queue;
mod viewport;

pub use self::content::TileContent;

use self::builder::TileBuilder;
use self::builder::TileParams;
pub use self::builder::TileType;
use self::queue::Queue;
use self::viewport::Viewport;
use crate::ColorScheme;
use tilemap::Task;
use tilemap::TileMap;

// We are blending the textures
pub const PADDING: u32 = 1;
pub const TEXTURE_SIZE: usize = 64 * 2;

/// Something that can build textures from tile pixels
// TODO: this is very ugly, kindly remove this
pub trait TileTextureProvider {
    type Texture;
    fn alloc(&mut self, pixels_rgba: &[u8]) -> Self::Texture;
    fn free(&mut self, texture: Self::Texture);
}

// TODO: uuugh anothher stuct named `Builder`, rename it or whatever
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

// main < done(p, px) < distributor
// main > params      > distributor
// main > viewport    > distributor
// distributor > params > worker
// distributor < status < worker
// distributor > pos    > worker

pub type TaskMap = TileMap<Task<TileContent>>;

/// After so many updates, i am not entierly sure what this struct is supposed to become
// TODO: use microserde? but we need derives
#[derive(Serialize, Deserialize)]
pub struct Fractal<T> {
    // state
    // NOTE: pos is public, so no need to forward its methods
    pub pos: Viewport,
    pub params: TileParams,
    pub clear: bool,

    // this uses a workaround to prevent incorrect `T: Default` bounds.
    // see: https://github.com/serde-rs/serde/issues/1541
    // TODO: maybe go back to locks?, i want to be able to clear a channel, that is not possible
    // as far as i know, also we have to be able to select when to recieve a position
    // TODO: params contain a version number
    #[serde(skip, default = "TileMap::new")]
    pub tiles: TileMap<T>,

    #[serde(skip, default = "TileMap::new")]
    pub tasks: TaskMap,

    #[serde(skip, default = "Builder::new")]
    pub builder: Builder,
}

impl<T> Fractal<T> {
    pub fn new(size: Vector2<u32>) -> Self {
        Fractal {
            tiles: TileMap::new(),
            tasks: TaskMap::new(),
            pos: Viewport::new(size),
            builder: Builder::new(),
            clear: false,
            params: TileParams {
                kind: TileType::Mandelbrot,
                iterations: 64,
                padding: PADDING,
                resolution: TEXTURE_SIZE as u32,
                color: ColorScheme::new(),
            },
        }
    }

    pub fn reload(&mut self) {
        self.clear = true;
    }

    pub fn distributor(&mut self) {
        // create new tiles
        let new_iter = self.pos.get_pos_all();
        self.tasks
            .update_with(new_iter, |_, _| (), |_| Some(Task::Todo));

        // Send tiles to builder
        // TODO: move to distributor
        for (p, t) in self.tasks.iter_mut() {
            match t {
                Task::Todo => match self.builder.queue.try_send(self.params.clone(), *p) {
                    Ok(_) => {
                        *t = Task::Doing;
                        println!("send");
                    },
                    Err(_) => break,
                },
                Task::Doing => (),
                Task::Done(_) => (),
            }
        }
    }

    pub fn update_tiles(&mut self, texture_creator: &mut impl TileTextureProvider<Texture = T>) {
        if self.clear {
            let tiles = std::mem::replace(&mut self.tiles, TileMap::new());
            for (_, t) in tiles.tiles.into_iter() {
                texture_creator.free(t);
            }
            self.tasks.clear();
            self.clear = false;
        }

        self.distributor();

        // read from builders
        while let Ok(r) = self.builder.queue.try_recv(&self.params) {
            println!("recev");
            if let Some((p, t)) = r {
                println!("ok");
                let t = texture_creator.alloc(&t.pixels);
                self.tiles.tiles.insert(p, t);
            }
        }

        // Free textures
        let new_iter = self.pos.get_pos_all();
        self.tiles
            .update_with(new_iter, |_, v| texture_creator.free(v), |_| None);
    }
}
