//! A service that can produce fractal image tiles. This trait should only
//! represent the method of computing and precision. Everything else is
//! determined by the arguments to the generate function.
//!
//! TODO: This should eventually become a trait.
//! Future implementations could be:
//!
//! TODO: switch between float/double depending on zoom level
//! TODO: dynamically check float/double availiability on ocl/cuda
//! 1) builders give a max depth and performance metric (but how do they check the params before
//!    building a tile?, there is no peeking a mpmc channel)
//!
//! INFO:
//!     cuda: cudaStreamQuery
//!
//! TODO: ablility to save and load multiple locations
//! TODO: share saved location (via clipboard)
//! TODO: when loading a location, we could transition to it, by first zooming out, then zooming in to that location
//! TODO: share sets of locations?
//!
//! TODO: the user should be able to change the algorithm.
//! This could be achieved with decent performance by doing each stage on
//! all pixels at the same time. this way the use of sse instructions
//! can be maximized and the number of comparisons minimized
//! Tile size should probably be configurable by the generator backend
//! implementations. As different backends have different optimal tile
//! sizes. (this was attemted but it was not very fast at all, also it is very
//! hard to use sse for this because each pixel in a tile needs a different iteration count)
//!
//! TODO: The colorscheme should be changable
//! TODO: 1) don't return exact pixels, but the complex numbers and/or iterations, same stuff.
//!       best would be to just return array of iteration count for each pixel. this way we also
//!       don't need to rebuild the tiles when changing the colorscheme
//!       also this would solve the horrible 'pixels -> implenetation spesific texture format'
//!       trait/functions passed to Fractal
//! TODO: 2) pass a colorscheme struct/trait via TileProperties. A colorscheme should be like a
//!       function of (iter%1.0,  iter/max_iter) -> color, this cold be encoded as a lookup image
//!
//! Why 1: the 'fractal' part of the library becomes a lot simpler
//! Why 2: possibly better performance

pub mod cpu;

#[cfg(feature = "builder-ocl")]
pub mod ocl;

use crate::fractal::queue::QueueHandle;
use crate::fractal::queue::TileResponse;
use crate::fractal::TileContent;
use crate::state::Reload;
use crate::ColorScheme;
use serde::{Deserialize, Serialize};
use std::thread::JoinHandle;
use tilemap::TilePos;

pub trait IsTileBuilder {
    fn configure(&mut self, p: &TileParams) -> bool;
    fn build(&mut self, p: TilePos) -> TileContent;
}

#[derive(Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Serialize, Deserialize, Debug)]
pub enum TileType {
    /// Used mostly for debugging
    Empty,
    /// ```text
    /// z = z ^ 2 + c
    /// ```
    Mandelbrot,
    /// Looks like a ship that is burning.
    /// ```text
    /// z = |re(z)| - |im(z)|i
    /// z = z^2 + c
    /// ```
    BurningShip,
    /// Very interesting fractal, burning ship + mandel3
    /// ```text
    /// z = |re(z)| - |im(z)|i
    /// z = z^2 + c
    /// z = z^3 + c
    /// ```
    ShipHybrid,
}

impl TileType {
    /// cycle between tiletypes
    pub fn next(self) -> Self {
        match self {
            TileType::Empty => TileType::Mandelbrot,
            TileType::Mandelbrot => TileType::BurningShip,
            TileType::BurningShip => TileType::ShipHybrid,
            TileType::ShipHybrid => TileType::Empty,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TileParamsSave {
    kind: TileType,
    iterations: i32,
}

impl Reload for TileParams {
    type Storage = TileParamsSave;

    fn load(&mut self, data: TileParamsSave) {
        self.kind = data.kind;
        self.iterations = data.iterations;
    }

    fn save(&self) -> TileParamsSave {
        TileParamsSave {
            kind: self.kind,
            iterations: self.iterations,
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct TileParams {
    pub kind: TileType,
    pub iterations: i32,

    // TODO: pub padding: f64? but then it cannot be Ord and has to be moved out of TilePos
    // We are not storing these, as they are constants and not somehting that should be stored
    pub resolution: u32,
    pub padding: u32,
    pub color: ColorScheme,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct TileRequest {
    pub pos: TilePos,
    pub version: usize,
    pub params: TileParams,
}

pub struct TileBuilder {
    handle: QueueHandle,

    #[allow(dead_code)]
    workers: Vec<JoinHandle<()>>,
}

impl TileBuilder {
    pub fn new(handle: QueueHandle) -> Self {
        let mut me = TileBuilder {
            handle,
            workers: Vec::new(),
        };

        #[cfg(feature = "builder-threaded")]
        {
            let ncpu = (num_cpus::get() - 1).max(1);
            for _ in 0..ncpu {
                me.add_builder(self::cpu::CPUBuilder::new());
            }
        }

        #[cfg(feature = "builder-ocl")]
        {
            use self::ocl::OCLWorker;
            if let Ok(w) = OCLWorker::new() {
                me.add_builder(w);
            }
        }

        me
    }

    pub fn add_builder<T: IsTileBuilder + Send + 'static>(&mut self, mut b: T) {
        let handle = self.handle.clone();
        self.workers.push(std::thread::spawn(move || loop {
            let mut version = 0;
            let mut active = false;
            loop {
                let h = match handle.tiles.upgrade() {
                    Some(h) => h,
                    None => break,
                };

                let mut h = h.lock();

                if h.params_version != version {
                    active = b.configure(&h.params);
                    version = h.params_version;
                }

                if !active {
                    drop(h);
                    handle.wait();
                    continue;
                }

                let next = match h.recv() {
                    None => {
                        drop(h);
                        handle.wait();
                        continue;
                    },
                    Some(next) => next,
                };

                // make sure the lock is freed before building
                drop(h);

                // do build
                let tile = b.build(next);

                let ret = handle.send(TileResponse {
                    pos: next,
                    version,
                    content: tile,
                });

                if ret.is_err() {
                    break;
                }
            }
        }));
    }
}
