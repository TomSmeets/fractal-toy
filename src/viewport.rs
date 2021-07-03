use crate::util::*;
use crate::TilePos;

#[derive(Debug)]
pub struct Viewport {
    pub zoom: f64,
    pub scale: f64,
    pub offset: V2,
    pub size_in_pixels: V2,
    pub size_in_pixels_i: V2<u32>,
    pub move_vel: V2,
    pub zoom_vel: f64,

    pub drag_anchor: Option<V2<f64>>,
    pub did_drag: bool,
}

impl Viewport {
    pub fn new() -> Self {
        Viewport {
            zoom: 0.,
            scale: 0.,
            size_in_pixels: V2::zero(),
            size_in_pixels_i: V2::zero(),
            offset: V2::zero(),
            move_vel: V2::zero(),
            zoom_vel: 0.0,

            drag_anchor: None,
            did_drag: false,
        }
    }

    pub fn size(&mut self, resolution: V2<u32>) {
        self.size_in_pixels = resolution.map(|x| x as f64);
        self.size_in_pixels_i = resolution;
    }

    pub fn zoom_at(&mut self, amount: f64, target: V2<i32>) {
        if amount * amount < 1e-6 {
            return;
        }

        let target_world = self.screen_to_world(target);
        self.zoom_center(amount);
        let current_world = self.screen_to_world(target);
        self.offset += target_world - current_world;
    }

    pub fn zoom_center(&mut self, amount: f64) {
        let amount = amount * 0.1;
        self.zoom_vel += amount;
        self.zoom += amount;
        self.scale = 0.5_f64.powf(self.zoom);
    }

    pub fn drag(&mut self, mouse: V2<i32>) {
        let current = self.screen_to_world(mouse);

        match self.drag_anchor {
            None => {
                self.drag_anchor = Some(current);
            },

            Some(target) => {
                self.offset -= current;
                self.offset += target;
                self.move_vel = target - current;
            },
        };

        self.did_drag = true;
    }

    pub fn update(&mut self, dt: f64) {
        if self.drag_anchor.is_some() && !self.did_drag {
            self.drag_anchor = None;
        }

        if !self.did_drag {
            self.offset += self.move_vel;
            self.move_vel *= 1.0 - dt * 5.0;
        }

        // self.zoom += self.zoom_vel*0.1;
        // self.scale = 0.5_f64.powf(self.zoom);
        // self.zoom_vel *= 1.0 - 5.0*dt;

        self.offset.x = self.offset.x.min(3.0).max(-3.0);
        self.offset.y = self.offset.y.min(3.0).max(-3.0);
        self.did_drag = false;
    }

    pub fn world_to_screen_rect(&self, r: &Rect) -> Rect {
        let min = r.corner_min();
        let max = r.corner_max();

        let min = self.world_to_screen(min).map(|x| x as _);
        let max = self.world_to_screen(max).map(|x| x as _);

        Rect::min_max(min, max)
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
    pub fn get_pos_all(&self, pad: i64) -> impl Iterator<Item = TilePos> {
        let mut cache = Vec::new();

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

        for z in (z_min as u8)..(z_max as u8 + 1) {
            TilePos::between(min, max, z, pad, &mut cache);
        }

        cache.into_iter()
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
