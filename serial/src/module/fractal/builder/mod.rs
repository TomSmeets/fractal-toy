pub mod cpu;
pub mod queue;
pub mod threaded;

use crate::module::fractal::tile::TilePos;

pub enum TileType {
    /// Used mostly for debugging
    Empty,
    /// ```
    /// z = z ^ 2 + c
    /// ```
    Mandelbrot,
    /// Looks like a ship that is burning.
    /// ```
    /// z = |re(z)| - |im(z)|i
    /// z = z^2 + c
    /// ```
    BurningShip,
    /// Very interesting fractal, burning ship + mandel3
    /// ```
    /// z = |re(z)| - |im(z)|i
    /// z = z^2 + c
    /// z = z^3 + c
    /// ```
    ShipHybrid,
}

pub struct TileRequest {
    pub pos: TilePos,
    pub kind: TileType,
    pub iterations: i32,
}
