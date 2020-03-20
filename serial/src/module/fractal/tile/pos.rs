use crate::math::*;

#[derive(Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct TilePos {
    pub z: i8,
    pub x: u64,
    pub y: u64,
}

impl TilePos {
    pub fn root() -> TilePos {
        TilePos { x: 0, y: 0, z: 0 }
    }

    pub fn from_f64(p: Vector2<f64>, z: i8) -> TilePos {
        let s = (1_u64 << z) as f64;
        TilePos {
            x: (p.x * s) as u64,
            y: (p.y * s) as u64,
            z,
        }
    }

    pub fn parent(&mut self) {
        if self.z > 0 {
            self.x /= 2;
            self.y /= 2;
            self.z -= 1;
        }
    }

    pub fn child(&mut self, dx: u64, dy: u64) {
        assert!(self.z < 64);
        self.x = self.x * 2 + dx;
        self.y = self.y * 2 + dy;
        self.z += 1;
    }

    pub fn max_size(&self) -> u64 {
        1_u64 << self.z
    }

    pub fn tile_scale(&self) -> f64 {
        1.0 / self.max_size() as f64
    }

    pub fn to_f32(&self) -> [f32; 4] {
        let s = self.tile_scale() as f32;
        [self.x as f32 * s, self.y as f32 * s, s, s]
    }

    pub fn to_f64(&self) -> [f64; 3] {
        let s = self.tile_scale();
        [self.x as f64 * s, self.y as f64 * s, s]
    }

    // pub fn around_region(&self, sx: u64, sy: u64) -> Vec<TilePos> {
    // let mut xs = Vec::new();
    //
    // let max = 1 << self.z;
    //
    // let max_x = (self.x + sx).min(max);
    // let max_y = (self.y + sy).min(max);
    //
    // for y in self.y..max_y {
    // for x in self.x..max_x {
    // xs.push(TilePos {
    // x: x,
    // y: y,
    // z: self.z,
    // });
    // }
    // }
    // xs
    // }
    //
    // pub fn translate(&mut self, dx: i64, dy: i64) {
    // let max = 1 << self.z;
    // self.x = (self.x + dx).max(0).min(max);
    // self.y = (self.y + dy).max(0).min(max);
    // }
    //
    // pub fn around_volume(&self, sx: u64, sy: u64, sz: u64) -> Vec<TilePos> {
    // let mut xs = Vec::new();
    //
    // let mut p1 = self.clone();
    // let mut p2 = self.clone();
    //
    // p2.translate(sx, sy);
    //
    // while sz > 0 {
    // xs.append(&mut p.around_region(sx, sy));
    // p.parent();
    // }
    //
    // xs
    // }
}
