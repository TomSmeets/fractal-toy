use super::tile::TilePos;
use super::TEXTURE_SIZE;
use crate::math::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Viewport {
    pub zoom: f64,
    pub offset: V2,
    pub size_in_pixels: V2,
}

impl Viewport {
    pub fn new(size_in_pixels: Vector2<u32>) -> Self {
        Viewport {
            zoom: 0.,
            size_in_pixels: to_v2(size_in_pixels),
            offset: V2::zero(),
        }
    }

    pub fn resize(&mut self, size_in_pixels: Vector2<u32>) {
        self.size_in_pixels = to_v2(size_in_pixels);
    }

    pub fn world_to_screen(&self, mut p: V2) -> V2i {
        // offset is in world space
        p -= self.offset;

        // y / vp_width
        p /= self.pixel_scale();

        // flip y
        p.y *= -1.0;

        // make center of screen  0,0
        p.x += self.size_in_pixels.x / 2.0;
        p.y += self.size_in_pixels.y / 2.0;

        Vector2::new(p.x as i32, p.y as i32)
    }

    pub fn screen_to_world(&self, p: V2i) -> V2 {
        let mut p = V2::new(p.x as f64, p.y as f64);

        // make center of screen 0,0
        p.x -= self.size_in_pixels.x / 2.0;
        p.y -= self.size_in_pixels.y / 2.0;

        // unflip y
        p.y *= -1.0;

        // normalize pixel coordinates

        // zoom
        p *= self.pixel_scale();

        // offset is in world space
        p += self.offset;

        p
    }

    /// Apply a translation in pixels
    pub fn translate(&mut self, offset: V2i) {
        let mut offset = to_v2(offset);
        offset.y *= -1.0;
        offset *= self.pixel_scale();
        self.offset += offset;
        self.offset.x = self.offset.x.min(3.0).max(-3.0);
        self.offset.y = self.offset.y.min(3.0).max(-3.0);
    }

    /// Zoom in or out towards a screen position
    pub fn zoom_in_at(&mut self, amount: f64, screen_pos: V2i) {
        if amount * amount < 0.001 {
            return;
        }

        let diff_in_pixels = screen_pos - to_v2i(self.size_in_pixels * 0.5);
        self.translate(diff_in_pixels);
        self.zoom_in(amount);
        self.translate(-diff_in_pixels);
    }

    /// Zoom in or out towards the screen center
    /// see also: [`Viewport::zoom_in_at`]
    pub fn zoom_in(&mut self, amount: f64) {
        self.zoom = (self.zoom + amount).min(48.5).max(-2.5);
    }

    /// scale of the entire viewport
    pub fn scale(&self) -> f64 {
        0.5_f64.powf(self.zoom)
    }

    /// scale of one pixel
    pub fn pixel_scale(&self) -> f64 {
        self.scale() / self.size_in_pixels.x
    }

    /// Returns an iterator with sorted tiles, the ordering is the same according to
    /// the ord implementation for TilePos
    pub fn get_pos_all(&self) -> impl Iterator<Item = TilePos> {
        // size of single pixel:
        // scale is width of entire viewport in the world
        //
        // px_size = (scale / with_in_pixels)
        // tile_size = px_size * TEXTURE_SIZE;
        // tile_size = (0.5)^z
        // z = log(tile_size)/log(1/2)
        // z = -log2(tile_size)
        let px_size = self.pixel_scale();
        let tile_size = px_size * TEXTURE_SIZE as f64;
        let z_max = -tile_size.log2();
        let z_max = z_max.max(0.0).ceil() as i32;
        let z_min = (z_max - 8).max(0);

        // extra padding in poportion to tile size
        let pad = 1;
        let off = self.offset;
        let s = 0.5 * px_size * self.size_in_pixels;

        (z_min as u8..z_max as u8 + 1).flat_map(move |z| {
            let min = off - s;
            let max = off + s;
            let min = TilePos::from_real(min.x, min.y, z);
            let max = TilePos::from_real(max.x, max.y, z);
            (min.x - pad..max.x + pad + 1)
                .flat_map(move |x| (min.y - pad..max.y + pad + 1).map(move |y| TilePos { x, y, z }))
        })
    }

    pub fn pos_to_rect(&self, p: &TilePos) -> Rect {
        fn mk_rect(a: V2i, b: V2i) -> Rect {
            let min_x = a.x.min(b.x);
            let min_y = a.y.min(b.y);

            let max_x = a.x.max(b.x);
            let max_y = a.y.max(b.y);

            let width = max_x.saturating_sub(min_x);
            let height = max_y.saturating_sub(min_y);

            Rect {
                pos: V2i::new(min_x, min_y),
                size: V2i::new(width, height),
            }
        }

        let [x, y, z] = p.to_f64();
        let min = V2::new(x, y);
        let max = min + V2::new(z, z);
        let min = self.world_to_screen(min);
        let max = self.world_to_screen(max);
        mk_rect(min, max)
    }
}

#[test]
fn test_viewport_pos_sorted() {
    let v = Viewport::new(Vector2::new(800, 600));
    let xs: Vec<_> = v.get_pos_all().collect();
    let mut ys = xs.clone();
    ys.sort();
    assert_eq!(xs, ys);
}