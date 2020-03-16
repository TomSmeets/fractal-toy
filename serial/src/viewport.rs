use crate::fractal::tile::TilePos;
use crate::math::*;

pub struct Viewport {
    pub zoom: f32,
    pub offset: Vector2<f32>,
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

    pub fn zoom_in(&mut self, amount: f32, view_pos: V2) {
        self.offset += self.scale() * view_pos;
        self.zoom += amount;
        self.offset -= self.scale() * view_pos;
    }

    pub fn scale(&self) -> f32 {
        0.5_f32.powf(self.zoom)
    }

    pub fn inv_scale(&self) -> f32 {
        2.0_f32.powf(self.zoom)
    }

    pub fn get_pos(&self) -> TilePos {
        let s = self.scale() / 2.0;
        TilePos::from_f32(self.offset + Vector2::new(s, s), (self.zoom + 2.0) as i8)
    }

    pub fn get_pos_all(&self) -> Vec<TilePos> {
        let z = (self.zoom + 2.5) as i8;
        let s = self.scale();

        let min = TilePos::from_f32(self.offset, z);
        let max = TilePos::from_f32(self.offset + Vector2::new(s, s), z);

        let mut v = Vec::new();
        for x in min.x..=max.x {
            for y in min.y..=max.y {
                v.push(TilePos { x, y, z });
            }
        }

        v
    }
}
