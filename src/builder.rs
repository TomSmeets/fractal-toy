use crate::util::*;
use crate::tilemap::TilePos;
use crate::Image;

use std::collections::BTreeMap;
use crossbeam_channel::{Sender, bounded};
use crossbeam_channel::Receiver;

pub struct TileBuilder {
    cache: BTreeMap<TilePos, Option<(Image, u32)>>,

    sender: Sender<(TilePos, V2)>,
    receiver: Receiver<(TilePos, Image)>,
}

impl TileBuilder {
    pub fn new() -> TileBuilder {
        let (req_send,  req_recv)  = bounded(16);
        let (tile_send, tile_recv) = bounded(16);

        for _ in 0..12 {
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

    pub fn gen_tile(p: &TilePos, a: V2) -> Image {
        const iter_count: usize = 1024;
        let iter_count_f = iter_count as f32;


        // the sin() and log2() can be optimized
        let size = 256;
        let mut data = Vec::with_capacity(size as usize * size as usize * 4);

        let pos = p.square();

        let min = pos.corner_min();
        let max = pos.corner_max();

        let center = min*0.5 + max*0.5;

        let min_r = min - a;
        let max_r = max - a;

        let mut anchor = a;
        let mut anchor_dist = f64::INFINITY;

        let c_big = a;
        let mut z_big_array = [[V2::zero(); 2]; iter_count];

        {
            let mut z = V2::zero();
            let c = c_big;
            for i in 0..iter_count {
                z_big_array[i][0].x = z.x as f32;
                z_big_array[i][0].y = z.y as f32;
                z_big_array[i][1].x = (z.x - z_big_array[i][0].x as f64) as f32;
                z_big_array[i][1].y = (z.y - z_big_array[i][0].y as f64) as f32;

                z = V2::new(
                    z.x*z.x - z.y*z.y,
                    2.0*z.x*z.y
                ) + c;
            }
        }

        for y in 0..size {
            for x in 0..size {
                let border = (y == 0 || y == size  - 1) || (x == 0 || x == size-1);

                let px = (x as f32) / size as f32;
                let py = (y as f32) / size as f32;

                let x = min_r.x as f32 * (1.0 - px) + max_r.x as f32 * px;
                let y = min_r.y as f32 * (1.0 - py) + max_r.y as f32 * py;

                let pi3 = std::f32::consts::FRAC_PI_3;


                let c_rel = V2::new(x, y);
                let c = c_rel + V2::new(a.x as f32, a.y as f32);

                let mut z = V2::zero();
                let mut t = 0.0;

                let c2 = c.x*c.x + c.y*c.y;

                // skip computation inside M1 - http://iquilezles.org/www/articles/mset_1bulb/mset1bulb.htm
                let in_m1 = 256.0*c2*c2 - 96.0*c2 + 32.0*c.x - 3.0 < 0.0;

                // skip computation inside M2 - http://iquilezles.org/www/articles/mset_2bulb/mset2bulb.htm
                let in_m2 = 16.0*(c2+2.0*c.x+1.0) - 1.0 < 0.0;

                let mut escape = false;
                // if in_m1 || in_m2 {
                //     t = iter_count_f - 1.0;
                // } else
                let mut d_tot = 0.0;
                {
                    for i in 0..iter_count {
                        let z_big  = z_big_array[i][0];
                        let z_big1 = z_big_array[i][1];

                        // 2*z_n*(Z_n + Z2_n)
                        // 2*z_n*Z_n + 2*z_n*Z2_n
                        let zz_x = 2.0*(z.x*z_big.x - z.y*z_big.y);
                        let zz_y = 2.0*(z.x*z_big.y + z.y*z_big.x);

                        let zz_1x = 2.0*(z.x*z_big1.x - z.y*z_big1.y);
                        let zz_1y = 2.0*(z.x*z_big1.y + z.y*z_big1.x);

                        z = V2::new(
                            z.x*z.x - z.y*z.y + zz_x + zz_1x,
                            2.0*z.x*z.y       + zz_y + zz_1y,
                        ) + c_rel;


                        let d = z.x*z.x + z.y*z.y;
                        d_tot += d;
                        if d > 256.0 {
                            t += -d.log2().log2() + 4.0;
                            // t = (a - c).magnitude() * 10.0 / p.tile_scale();
                            escape = true;
                            break;
                        }
                        t += 1.0;
                    }
                }

                if border {
//                    t = 1.0
                }

                if !escape {
                    let d1 = (a + z.cast().unwrap()).magnitude2();
                    if d1 < anchor_dist {
                        let c = a + c_rel.cast().unwrap();
                        anchor = c;
                        anchor_dist = d1;
                        t = 0.0;
                    }
                }

                let a = (1.0 - (t/(1024.0)).powi(2)).min(1.0).max(0.0);
                let t = t*0.1;
                let r = a * ((0.5 - t)*3.0*pi3 + pi3*0.0).sin();
                let g = a * ((0.5 - t)*3.0*pi3 + pi3*1.0).sin();
                let b = a * ((0.5 - t)*3.0*pi3 + pi3*2.0).sin();

                let r = r*r;
                let g = g*g;
                let b = b*b;

                data.push((r * 255.0) as _);
                data.push((g * 255.0) as _);
                data.push((b * 255.0) as _);
                data.push(255);
            }
        }

        Image { size: V2::new(size, size), data, anchor }
    }

    // pub fn tile(&mut self, p: &TilePos) -> Option<&Image> {
    //     // check cache
    //     // build
    //     // find refernce point
    //     let parent = self.tile(&p.parent().unwrap()).unwrap();
    //     
    // }


    pub fn tile(&mut self, p: &TilePos) -> Option<Image> {

        // Check cache
        if let Some(cache_entry) = self.cache.get_mut(p) {
            match cache_entry {
                // The tile was cached
                Some((img, count)) => {
                    *count += 1;
                    return Some(img.clone());
                },

                // The tile is already queud, just not done yet
                None => return None,
            };
        }

        // Check parent for an anchor
        let anchor = match p.parent() {
            Some(p) => match self.tile(&p) {
                // Parent is cached
                Some(t) => t.anchor,

                // Parent is not done yet, all we can do is wait
                None => return None,
            },

            // This tile is the root tile, default to 0,0 as anchor
            None => V2::new(0.0, 0.0),
        };

        // tell a builder to build this tile
        if let Ok(_) = self.sender.try_send((*p, anchor)) {
            // Tile is queued, don't request it again
            self.cache.insert(*p, None);
        }

        None
    }

    pub fn update(&mut self) {
        let mut new_cache = BTreeMap::new();

        dbg!(self.cache.len());
        for (k, v) in std::mem::take(&mut self.cache) {
            match v {
                Some((img, cnt)) if cnt > 0 => { new_cache.insert(k, Some((img, 0))); },
                None => { new_cache.insert(k, None); } ,
                _ => (),
            };
        }

        // TODO: clear cache?

        // Check for finished tiles
        while let Ok((p, i)) = self.receiver.try_recv() {
            new_cache.insert(p, Some((i, 1)));
        }

        self.cache = new_cache;
    }
}

