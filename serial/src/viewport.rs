use crate::fractal::tile::TilePos;
use crate::math::*;

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
        self.offset += self.scale() * view_pos;
        self.zoom += amount;
        self.offset -= self.scale() * view_pos;
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

    pub fn get_pos_all(&self) -> Vec<TilePos> {
        let z_min = (self.zoom - 5.5).max(0.0) as i8;
        let z_max = (self.zoom + 5.5) as i8;
        let s = self.scale();

        let mut v = Vec::new();
        for z in z_min..=z_max {
            let min = TilePos::from_f64(self.offset, z);
            let max = TilePos::from_f64(self.offset + Vector2::new(s, s), z);

            for x in min.x..=max.x {
                for y in min.y..=max.y {
                    v.push(TilePos { x, y, z });
                }
            }
        }

        v
    }
}
