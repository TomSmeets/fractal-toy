//! A service that can produce fractal image tiles. This trait should only
//! represent the method of computing and precision. Everything else is
//! determined by the arguments to the generate function.

// TODO: switch between float/double depending on zoom level
// TODO: dynamically check float/double availiability on ocl/cuda
// TODO: ablility to save and load multiple locations
// TODO: share saved location (via clipboard)
// TODO: when loading a location, we could transition to it, by first zooming out, then zooming in to that location
// TODO: share sets of locations?
// TODO: The user should be able to change the algorithm.
//       This could be achieved with decent performance by doing each stage on
//       all pixels at the same time. this way the use of sse instructions
//       can be maximized and the number of comparisons minimized
//       Tile size should probably be configurable by the generator backend
//       implementations. As different backends have different optimal tile
//       sizes. (this was attemted but it was not very fast at all, also it is very
//       hard to use sse for this because each pixel in a tile needs a different iteration count)
//       pass a colorscheme struct/trait via TileProperties. A colorscheme should be like a
//       function of (iter%1.0,  iter/max_iter) -> color, this cold be encoded as a lookup image
//
// TODO: The colorscheme should be changable
// TODO: The coloring method should be changable (orbit trap)

use crate::state::Reload;
use crate::TextureSizeAndPadding;
use serde::{Deserialize, Serialize};
use tilemap::TilePos;

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
    pub size: TextureSizeAndPadding,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct TileRequest {
    pub pos: TilePos,
    pub version: usize,
    pub params: TileParams,
}
