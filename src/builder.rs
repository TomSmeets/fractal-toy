use crate::tilemap::TilePos;
use crate::util::*;
use crate::Image;

use crossbeam_channel::Receiver;
use crossbeam_channel::{bounded, Sender};
use std::collections::BTreeMap;

const ITER_COUNT: usize = 1024;

pub struct TileBuilder {
    cache: BTreeMap<TilePos, Option<(Image, u32)>>,

    sender: Sender<(TilePos, V2)>,
    receiver: Receiver<(TilePos, Image)>,
}

impl TileBuilder {
    pub fn new() -> TileBuilder {
        let (req_send, req_recv) = bounded(16);
        let (tile_send, tile_recv) = bounded(16);

        for _ in 0..6 {
            let tile_send = tile_send.clone();
            let req_recv = req_recv.clone();
            std::thread::spawn(move || {
                while let Ok((pos, a)) = req_recv.recv() {
                    tile_send.send((pos, Self::gen_tile(&pos, a))).unwrap();
                }
            });
        }

        TileBuilder {
            cache: BTreeMap::new(),
            sender: req_send,
            receiver: tile_recv,
        }
    }

    fn calculate_refernce_with(c: V2) -> [[V2<f32>; 2]; ITER_COUNT] {
        let mut z_values = [[V2::zero(); 2]; ITER_COUNT];
        let mut z = V2::zero();
        for i in 0..ITER_COUNT {
            z_values[i][0].x = z.x as f32;
            z_values[i][0].y = z.y as f32;

            // NOTE: does this even work?, also does it help
            z_values[i][1].x = (z.x - z_values[i][0].x as f64) as f32;
            z_values[i][1].y = (z.y - z_values[i][0].y as f64) as f32;

            z = V2::new(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y) + c;
        }

        z_values
    }

    fn gen_tile(p: &TilePos, a: V2) -> Image {
        // the sin() and log2() can be optimized
        let size = 256;
        let mut data = Vec::with_capacity(size as usize * size as usize * 4);

        let pos = p.square();

        let min = pos.corner_min();
        let max = pos.corner_max();

        let center = min * 0.5 + max * 0.5;

        for y in 0..size {
            for x in 0..size {
                let border = (y == 0 || y == size - 1) || (x == 0 || x == size - 1);

                let px = (x as f64 + 0.5) / (size) as f64;
                let py = (y as f64 + 0.5) / (size) as f64;

                let x = min.x as f64 * (1.0 - px) + max.x as f64 * px;
                let y = min.y as f64 * (1.0 - py) + max.y as f64 * py;

                let pi3 = std::f64::consts::FRAC_PI_3;

                let c = V2::new(x, y);

                let mut z: V2<f64> = V2::zero();
                let mut t = 0.0;

                let c2 = c.x * c.x + c.y * c.y;

                // skip computation inside M1 - http://iquilezles.org/www/articles/mset_1bulb/mset1bulb.htm
                let in_m1 = 256.0 * c2 * c2 - 96.0 * c2 + 32.0 * c.x - 3.0 < 0.0;

                // skip computation inside M2 - http://iquilezles.org/www/articles/mset_2bulb/mset2bulb.htm
                let in_m2 = 16.0 * (c2 + 2.0 * c.x + 1.0) - 1.0 < 0.0;

                // if in_m1 || in_m2 {
                //     t = ITER_COUNT as f64 - 1.0;
                // } else {
                for i in 0..ITER_COUNT {
                    // z = V2 {
                    //     x: z.x * z.x * z.x - 3.0 * z.x * z.y * z.y + c.x,
                    //     y: 3.0 * z.x * z.x * z.y - z.y * z.y * z.y + c.y,
                    // };
                    z = V2 {
                        x: z.x * z.x - z.y * z.y + c.x,
                        y: 2.0 * z.x * z.y + c.y,
                    };

                    z.y = -z.y.abs();
                    z.x = z.x.abs();


                    let d = z.x * z.x + z.y * z.y;
                    if d > 256.0 {
                        t += -d.log2().log2() + 4.0;
                        break;
                    }
                    t += 1.0;
                }
                // }

                // if border {
                //     t = 1.0
                // }

                let a = (1.0 - (t / (1024.0)).powi(2)).min(1.0).max(0.0);
                let t = t * 0.005;
                let r = a * ((0.5 - t) * 3.0 * pi3 + pi3 * 0.0).sin();
                let g = a * ((0.5 - t) * 3.0 * pi3 + pi3 * 1.0).sin();
                let b = a * ((0.5 - t) * 3.0 * pi3 + pi3 * 2.0).sin();

                let r = r * r;
                let g = g * g;
                let b = b * b;

                data.push((r * 255.0) as _);
                data.push((g * 255.0) as _);
                data.push((b * 255.0) as _);
                data.push(255);
            }
        }

        Image::new(V2::new(size, size), data)
    }

    /// Either return a cached tile, or add it to the build queue
    pub fn tile(&mut self, p: &TilePos) -> Option<&Image> {
        let in_cache = self.cache.contains_key(p);

        if !in_cache {
            // tell a builder to build this tile
            if let Ok(_) = self.sender.try_send((*p, V2::zero())) {
                // Tile is queued, don't request it again
                self.cache.insert(*p, None);
            }

            None
        } else {
            let cache_entry = self.cache.get_mut(p).unwrap();
            match cache_entry {
                // The tile was cached
                Some((img, count)) => {
                    *count += 1;
                    return Some(img);
                },

                // The tile is already queud, just not done yet
                None => return None,
            };
        }
    }

    /// update the cache, removing unused tiles and inserting newly finished tiles
    pub fn update(&mut self) {
        let mut new_cache = BTreeMap::new();

        for (k, v) in std::mem::take(&mut self.cache) {
            match v {
                Some((img, cnt)) if cnt > 0 => {
                    new_cache.insert(k, Some((img, 0)));
                },
                None => {
                    new_cache.insert(k, None);
                },
                _ => (),
            };
        }

        // Check for finished tiles
        while let Ok((p, i)) = self.receiver.try_recv() {
            new_cache.insert(p, Some((i, 1)));
        }

        self.cache = new_cache;
    }
}
