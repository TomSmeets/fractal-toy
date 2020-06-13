//! A service that can produce fractal image tiles. This trait should only
//! represent the method of computing and precision. Everything else is
//! determined by the arguments to the generate function.
//!
//! TODO: This should eventually become a trait.
//! Future implementations could be:
//!
//! TODO: cuda-float
//! TODO: cuda-double
//! TODO: opencl-float
//! TODO: opencl-double
//! TODO: cpu-float
//! TODO: cpu-double
//! TODO: sse2
//! TODO: avx
//!
//! INFO:
//!     cuda: cudaStreamQuery
//!
//! TODO: ablility to save and load multiple locations
//!
//! TODO: share saved location (via clipboard)
//!
//! TODO: when loading a location, we could transition to it, by first zooming
//!       out, then zooming in to that location
//!
//! TODO: share sets of locations?
//!
//! TileBuilder:  threaded, cuda, opencl
//! TileType:     Empty, Mandel, BurningShip
//!
//! we have one queue, and ecach tilebuilder grabs from that queue
//! builders = [ CudaTileBuilder(Arc::clone(queue)),
//! ThrededBuidler(Arc::clone(queue)) ]
//!
//! then the queue contains all information for poducing a tile.
//! like iteration count, fractal type, etc
//!
//! NOTE:
//! the user should be able to change the algorithm.
//! This could be achieved with decent performance by doing each stage on
//! all pixels at the same time. this way the use of sse instructions
//! can be maximized and the number of comparisons minimized
//! Tile size should probably be configurable by the generator backend
//! implementations. As different backends have different optimal tile
//! sizes. (this was attemted but it was not very fast at all, also it is very
//! hard to use sse for this)
//!
//! NOTE: don't return excact pixels, but the complex
//! numbers and/or iterations, same stuff. best would be to just return array of
//! iteration count for each pixel. this would save. argb8: 4 bytes per pixel
//!
//! rgb8:  3 bytes per pixel
//! iter:  2 bytes per pixel (u16)
//! iter:  4 bytes per pixel (f32)
//! iter:  8 bytes per pixel (f64)
//!
//! so rgb is not that bad actually, we should drop the alpha component (what
//! about alignment? profile!)

pub mod cpu;

#[cfg(feature = "builder-threaded")]
pub mod threaded;

#[cfg(feature = "builder-ocl")]
pub mod ocl;

use crate::fractal::TileContent;
use crate::tilemap::TilePos;
use crossbeam_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};

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

#[derive(Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Serialize, Deserialize, Debug)]
pub struct TileParams {
    pub kind: TileType,
    pub iterations: i32,

    // TODO: pub padding: f64? but then it cannot be Ord and has to be moved out of TilePos
    pub resolution: u32,
    pub padding: u32,
}

#[derive(Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Debug)]
pub struct TileRequest {
    pub pos: TilePos,
    // TODO: move out of here to save some memory?
    pub params: TileParams,
}

pub struct TileBuilder {
    #[cfg(feature = "builder-threaded")]
    #[allow(dead_code)]
    threaded: self::threaded::ThreadedTileBuilder,

    #[cfg(feature = "builder-ocl")]
    #[allow(dead_code)]
    ocl: self::ocl::OCLTileBuilder,
}

impl TileBuilder {
    pub fn new(rx: Receiver<TileRequest>, tx: Sender<(TileRequest, TileContent)>) -> Self {
        TileBuilder {
            #[cfg(feature = "builder-threaded")]
            threaded: self::threaded::ThreadedTileBuilder::new(rx.clone(), tx.clone()),

            #[cfg(feature = "builder-ocl")]
            ocl: self::ocl::OCLTileBuilder::new(rx.clone(), tx.clone()),
        }
    }

    pub fn update(&mut self) {}
}
