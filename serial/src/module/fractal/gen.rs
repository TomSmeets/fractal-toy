use super::{TilePos, TEXTURE_SIZE};
use crate::math::*;
use serde::{Deserialize, Serialize};

/// A service that can produce fractal image tiles. This trait should only
/// represent the method of computing and precision. Everything else is
/// determined by the arguments to the generate function.
///
/// TODO: This should eventually become a trait.
/// Future implementations could be:
///     TODO: cuda-float
///     TODO: cuda-double
///     TODO: opencl-float
///     TODO: opencl-double
///     TODO: cpu-float
///     TODO: cpu-double
///     TODO: sse2
///     TODO: avx
///
/// INFO:
///     cuda: cudaStreamQuery
/// TODO: ablility to save and load multiple locations
/// TODO: share saved location (via clipboard)
/// TODO: when loading a location, we could transition to it, by first zooming
///       out, then zooming in to that location
/// TODO: share sets of locations?
#[derive(Serialize, Deserialize)]
pub struct Gen {
    iterations: u32,
}

/// TileBuilder:  threaded, cuda, opencl
/// TileType:     Empty, Mandel, BurningShip
///
/// we have one queue, and ecach tilebuilder grabs from that queue
/// builders = [ CudaTileBuilder(Arc::clone(queue)),
/// ThrededBuidler(Arc::clone(queue)) ]
///
/// then the queue contains all information for poducing a tile.
/// like iteration count, fractal type, etc
impl Gen {
    pub fn new() -> Gen {
        Gen { iterations: 256 }
    }

    /// This function should receive all required
    /// information to generate a reproducible fractal image
    ///
    /// TODO:
    /// the user should be able to change the algorithm.
    /// This could be achieved with decent performance by doing each stage on
    /// all pixels at the same time. this way the use of sse instructions
    /// can be maximized and the number of comparisons minimized
    /// Tile size should probably be configurable by the generator backend
    /// implementations. As different backends have different optimal tile
    /// sizes.
    /// TODO: don't return excact pixels, but the complex numbers and/or
    /// iterations
    /// TODO: This gen shold be initialized with fractal algorithm (and maybe
    /// colorcheme), so it can pre compile a program for it.
    pub fn generate(&self, tile: TilePos) -> Vec<u8> {
        let mut pixels = vec![0; (TEXTURE_SIZE * TEXTURE_SIZE * 4) as usize];

        let [x, y, size] = tile.to_f64_with_padding();
        let mut center = Vector2::new(x, y) * 4.0;

        if true {}

        if false {
            for y in 0..TEXTURE_SIZE {
                for x in 0..TEXTURE_SIZE {
                    if x <= 4 || y <= 4 || x >= TEXTURE_SIZE - 5 || y >= TEXTURE_SIZE - 5 {
                        pixels[0 + y * TEXTURE_SIZE * 4 + x * 4] = 64;
                        pixels[1 + y * TEXTURE_SIZE * 4 + x * 4] = 64;
                        pixels[2 + y * TEXTURE_SIZE * 4 + x * 4] = 64;
                        pixels[3 + y * TEXTURE_SIZE * 4 + x * 4] = 64;
                    }
                }
            }
        }

        pixels
    }
}
