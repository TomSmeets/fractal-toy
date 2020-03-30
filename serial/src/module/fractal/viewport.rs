use super::tile::TilePos;
use crate::math::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Viewport {
    pub zoom: f64,
    pub offset: Vector2<f64>,
}

impl Viewport {
    pub fn new() -> Self {
        Viewport {
            zoom: 0.,
            offset: V2::zero(),
        }
    }

    pub fn world_to_view(&self, p: V2) -> V2 {
        let scale_inv = self.inv_scale();
        V2::new(
            (p.x - self.offset.x) * scale_inv,
            (p.y - self.offset.y) * scale_inv,
        )
    }

    pub fn view_to_world(&self, p: V2) -> V2 {
        let scale = self.scale();
        V2::new((p.x) * scale + self.offset.x, (p.y) * scale + self.offset.y)
    }

    pub fn translate(&mut self, offset: V2) {
        self.offset += offset * self.scale();
    }

    pub fn zoom_in(&mut self, amount: f64, view_pos: V2) {
        if amount * amount > 0.001 {
            self.offset += self.scale() * view_pos;
            self.zoom = (self.zoom + amount).min(48.5).max(-0.5);
            self.offset -= self.scale() * view_pos;
        }
    }

    pub fn scale(&self) -> f64 {
        0.5_f64.powf(self.zoom)
    }

    pub fn inv_scale(&self) -> f64 {
        2.0_f64.powf(self.zoom)
    }

    pub fn get_pos(&self) -> TilePos {
        let s = self.scale() / 2.0;
        TilePos::from_f64(self.offset + Vector2::new(s, s), (self.zoom + 2.0) as i8)
    }

    /// should be sorted from z_low to z_high
    /// ordering is: z > y > x
    /// this should probably be the same as the ord implementation of TilePos
    /// ```rust
    /// use serial::module::fractal::viewport::Viewport;
    /// let v = Viewport::new();
    /// let xs: Vec<_> = v.get_pos_all().collect();
    /// let mut ys = xs.clone();
    /// ys.sort();
    /// assert_eq!(xs, ys);
    /// ```
    pub fn get_pos_all(&self) -> impl Iterator<Item = TilePos> {
        let z_min = (self.zoom - 5.5).max(0.0) as i8;
        let z_max = (self.zoom + 1.5) as i8;
        let s = self.scale();
        let off = self.offset;

        (z_min..=z_max).flat_map(move |z| {
            let min = TilePos::from_f64(off, z);
            let max = TilePos::from_f64(off + Vector2::new(s, s), z);
            (min.x..=max.x).flat_map(move |x| (min.y..=max.y).map(move |y| TilePos { x, y, z }))
        })
    }
}
