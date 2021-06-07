use crate::util::*;
use crate::tilemap::TilePos;
use crate::Image;

use std::collections::BTreeMap;
use crossbeam_channel::{Sender, bounded};
use crossbeam_channel::Receiver;

pub struct TileBuilder {
    cache: BTreeMap<TilePos, Image>,
    sender: Sender<(TilePos, V2)>,
    receiver: Receiver<(TilePos, Image)>,
}

impl TileBuilder {
    pub fn new() -> TileBuilder {
        let (req_send,  req_recv)  = bounded(16);
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

    pub fn gen_tile(p: &TilePos, a: V2) -> Image {
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
        let mut anchor_dist = 1000000.0;

        let c_big = a;
        let mut z_big = [V2::zero(); 1024];

        {
            let mut z = V2::zero();
            let c = c_big;
            for i in 0..1024 {
                z_big[i] = z;
                z = V2::new(
                    z.x*z.x - z.y*z.y,
                    2.0*z.x*z.y
                ) + c;
            }
        }

        for y in 0..size {
            for x in 0..size {
                let border = (y == 0 || y == size  - 1) || (x == 0 || x == size-1);

                let px = (x as f64 + 0.5) / size as f64;
                let py = (y as f64 + 0.5) / size as f64;

                let x = min_r.x as f64 *(1.0 - px) + max_r.x as f64 * px;
                let y = min_r.y as f64 *(1.0 - py) + max_r.y as f64 * py;

                let pi3 = std::f64::consts::FRAC_PI_3;


                let c_rel = V2::new(x, y);
                let c = c_rel + a;
                let mut z = V2::zero();
                let mut t = 0.0;

                let c2 = c.x*c.x + c.y*c.y;

                // skip computation inside M1 - http://iquilezles.org/www/articles/mset_1bulb/mset1bulb.htm
                let in_m1 = 256.0*c2*c2 - 96.0*c2 + 32.0*c.x - 3.0 < 0.0;

                // skip computation inside M2 - http://iquilezles.org/www/articles/mset_2bulb/mset2bulb.htm
                let in_m2 = 16.0*(c2+2.0*c.x+1.0) - 1.0 < 0.0;

                let mut escape = false;
                if in_m1 || in_m2 {
                    t = 255.0;
                } else {
                    for i in 0..1024 {
                        let z_big = z_big[i];

                        // 2*z_n*Z_n
                        let zz_x = 2.0*(z.x*z_big.x - z.y*z_big.y);
                        let zz_y = 2.0*(z.x*z_big.y + z.y*z_big.x);

                        z = V2::new(
                            z.x*z.x - z.y*z.y + zz_x,
                            2.0*z.x*z.y       + zz_y
                        ) + c_rel;

                        let d = z.x*z.x + z.y*z.y;
                        if d > 1024.0 {
                            t += -d.log2().log2() + 4.0;
                            // t = (a - c).magnitude() * 10.0 / p.tile_scale();
                            escape = true;
                            break;
                        }
                        t += 1.0;
                    }
                }

                if !escape {
                    let d1 = (center - c).magnitude2();
                    if d1 <= anchor_dist {
                        anchor = c;
                        anchor_dist = d1;
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

    pub fn build(&mut self, pos: &[TilePos]) -> &BTreeMap<TilePos, Image> {
        let mut old_cache = std::mem::take(&mut self.cache);
        let mut new_cache = BTreeMap::new();

        while let Ok((k, v)) = self.receiver.try_recv() {
            old_cache.insert(k, v);
        }

        let mut todo_count = 0;
        let mut done_count = 0;
        for p in pos {
            if let Some(v) = old_cache.remove(p) {
                new_cache.insert(*p, v);
                done_count += 1;
            } else {
                let anchor = match p.parent() {
                    Some(p) => old_cache.get(&p).or_else(|| new_cache.get(&p)).map(|x| x.anchor),
                    None => Some(V2::new(0.0, 0.0)),
                };

                if let Some(anchor) = anchor {
                    let _ = self.sender.try_send((*p, anchor));
                }
                todo_count += 1;
            }
        }

        // println!("todo: {}, done: {}", todo_count, done_count);

        self.cache = new_cache;
        &self.cache
    }
}

