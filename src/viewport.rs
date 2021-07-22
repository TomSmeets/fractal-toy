use crate::tilemap::TilePos;
use crate::util::*;

#[derive(Debug)]
pub struct Viewport {
    pub zoom: f64,
    pub scale: f64,
    pub offset: V2,
    pub size_in_pixels: V2,
    pub size_in_pixels_i: V2<u32>,
    pub move_vel: V2,
    pub drag_anchor: Option<V2<f64>>,
}

pub struct ViewportInput {
    pub dt: f64,
    pub resolution: V2<u32>,
    pub dir_move: V2<f64>,
    pub zoom_center: f64,
    pub scroll_at: (V2<i32>, f64),
    pub drag: Option<V2<i32>>,
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

            drag_anchor: None,
        }
    }

    pub fn update(&mut self, input: &ViewportInput) {
        self.size_in_pixels = input.resolution.map(|x| x as f64);
        self.size_in_pixels_i = input.resolution;

        match input.drag {
            Some(mouse) => {
                let mouse_world = self.screen_to_world(mouse);
                let target_world = *self.drag_anchor.get_or_insert(mouse_world);
                let diff = target_world - mouse_world;
                self.offset += diff;
                self.move_vel = diff * 1.0 / input.dt;
            }
            None => {
                self.drag_anchor = None;

                self.offset += input.dt * self.scale * input.dir_move;

                // velocity
                self.offset += input.dt * self.move_vel;
                self.move_vel *= 1.0 - input.dt * 5.0;
                if self.move_vel.magnitude2()
                    < (self.scale * input.dt) * (self.scale * input.dt) * 1e-6
                {
                    self.move_vel = V2::zero();
                }
            }
        }

        let mut scroll_world_pos = None;
        let (scroll_pos, scroll_amount) = input.scroll_at;
        if scroll_amount * scroll_amount > 1e-6 {
            self.zoom += scroll_amount * 0.1;
            scroll_world_pos = Some(self.screen_to_world(scroll_pos));
        }

        self.zoom += input.dt * input.zoom_center;

        self.offset.x = self.offset.x.min(3.0).max(-3.0);
        self.offset.y = self.offset.y.min(3.0).max(-3.0);

        // zooming in too far will result in overflows, we might go to 128 bit numbers?
        self.zoom = self.zoom.min(53.0).max(-4.0);
        self.scale = 0.5_f64.powf(self.zoom);

        if let Some(scroll_world_pos) = scroll_world_pos {
            let current_world_pos = self.screen_to_world(scroll_pos);
            self.offset += scroll_world_pos - current_world_pos;
        }
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
        self.scale
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
