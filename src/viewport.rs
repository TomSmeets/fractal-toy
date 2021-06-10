use crate::TilePos;
use crate::util::*;

pub struct ViewportInput {
    pub resolution: V2<u32>,
    // translate: V2,
    pub zoom: (f64, V2<i32>),

    pub world2screen: Option<(V2, V2<i32>)>,
}

#[derive(Debug)]
pub struct Viewport {
    zoom: f64,
    pub scale: f64,
    pub offset: V2,
    size_in_pixels: V2,
}

impl Viewport {
    pub fn new() -> Self {
        Viewport {
            zoom: 0.,
            scale: 0.,
            size_in_pixels: V2::zero(),
            offset: V2::zero(),
        }
    }

    pub fn update(&mut self, dt: f64, input: &ViewportInput) -> &Self {
        self.size_in_pixels = input.resolution.map(|x| x as f64);

        let w0 = self.screen_to_world(input.zoom.1);

        self.zoom += 0.1 * input.zoom.0;
        self.scale = 0.5_f64.powf(self.zoom);

        if let Some((w1, s)) = input.world2screen {
            let w2 = self.screen_to_world(s);

            // currently s is at w2, but should be at w1
            self.offset += w1 - w2;
        } else {
            let w2 = self.screen_to_world(input.zoom.1);
            self.offset += w0 - w2;
        }

        // offset.y *= -1.0;
        // offset *= self.pixel_size();

        // self.offset += dt*input.translate;
        self.offset.x = self.offset.x.min(3.0).max(-3.0);
        self.offset.y = self.offset.y.min(3.0).max(-3.0);

        self
    }

    pub fn world_to_screen(&self, mut p: V2) -> V2<i32> {
        // offset is in world space
        p -= self.offset;

        // y / vp_width
        p /= self.pixel_size();

        // flip y
        p.y *= -1.0;

        // make center of screen  0,0
        p.x += self.size_in_pixels.x / 2.0;
        p.y += self.size_in_pixels.y / 2.0;

        V2::new(p.x as i32, p.y as i32)
    }

    /// Convert a screen-space position to a world position as seen by this viewport
    pub fn screen_to_world(&self, p: V2<i32>) -> V2 {
        let mut p = V2::new(p.x as f64, p.y as f64);

        // make center of screen 0,0
        p.x -= self.size_in_pixels.x / 2.0;
        p.y -= self.size_in_pixels.y / 2.0;

        // unflip y
        p.y *= -1.0;

        // normalize pixel coordinates

        // zoom
        p *= self.pixel_size();

        // offset is in world space
        p += self.offset;

        p
    }

    /*
    /// Apply a translation in pixels
    pub fn translate(&mut self, offset: V2i) {
        let mut offset = to_v2(offset);
        offset.y *= -1.0;
        offset *= self.pixel_size();
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
    */

    /// scale of the entire viewport
    pub fn scale(&self) -> f64 {
        0.5_f64.powf(self.zoom)
    }

    /// The size of one pixel in world space
    pub fn pixel_size(&self) -> f64 {
        self.scale() / self.size_in_pixels.x
    }


    /// Returns an iterator with sorted tiles, the ordering is the same according to
    /// the ord implementation for TilePos
    pub fn get_pos_all(&self, dst: &mut Vec<TilePos>, pad: i64) {
        // size of single pixel:
        // scale is width of entire viewport in the world
        //
        // px_size = (scale / with_in_pixels)
        // tile_size = px_size * TEXTURE_SIZE;
        // tile_size = (0.5)^z
        // z = log(tile_size)/log(1/2)
        // z = -log2(tile_size)
        let px_size = self.pixel_size();
        let tile_size = px_size * 256.0 as f64;
        let z_max = -tile_size.log2();
        let z_max = z_max.max(0.0).ceil() as i32;
        let z_min = 0; // (z_max - 8).max(0);

        // extra padding in poportion to tile size
        let off = self.offset;
        let viewport_half_size = 0.5 * px_size * self.size_in_pixels;

        fn clamp(v: V2) -> V2 {
            V2 {
                x: v.x.min(2.9).max(-2.9),
                y: v.y.min(2.9).max(-2.9),
            }
        }

        let min = clamp(off - viewport_half_size);
        let max = clamp(off + viewport_half_size);

        for z in (z_min as u8) .. (z_max as u8 + 1) {
            TilePos::between(min, max, z, pad, dst);
        }
    }

    /*
    /// Convert a TilePos to a screen-space Rectangle as seen by this viewport
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

        let rect = p.square();
        let min = rect.corner_min();
        let max = rect.corner_max();
        let min = self.world_to_screen(min);
        let max = self.world_to_screen(max);
        mk_rect(min, max)
    }
    */
}

/*
#[test]
fn test_viewport_pos_sorted() {
    let mut v = Viewport::new();
    let v = v.update(1.0, &ViewportInput {
        resolution: V2::new(800, 600),
        translate: V2::zero(),
        zoom: 0.0,
        world2screen: None,
    });
    let xs: Vec<_> = v.get_pos_all().collect();
    let mut ys = xs.clone();
    ys.sort();
    assert_eq!(xs, ys);
}
*/
