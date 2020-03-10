use crate::math::*;
use crate::quadtree::pos::QuadTreePosition;

pub struct Viewport {
    zoom: f32,
    offset: Vector2<f32>,
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

    pub fn get_pos(&self) -> QuadTreePosition {
        let mut z = self.zoom + 1.0;
        let mut x = self.offset.x + 0.5 * self.scale();
        let mut y = self.offset.y + 0.5 * self.scale();
        let mut node = QuadTreePosition::root();

        while z > 0. {
            let qx = if x > 0.5 { 1 } else { 0 };
            let qy = if y > 0.5 { 1 } else { 0 };
            node.child(qx, qy);
            z -= 1.;
            x = x * 2.0 - qx as f32;
            y = y * 2.0 - qy as f32;
        }

        node
    }
}
