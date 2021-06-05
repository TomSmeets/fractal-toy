use crate::util::*;
use crate::tilemap::TilePos;
use crate::Image;

use std::collections::BTreeMap;
use crossbeam_channel::{Sender, bounded};
use crossbeam_channel::Receiver;

pub struct TileBuilder {
    cache: BTreeMap<TilePos, Image>,
    sender: Sender<TilePos>,
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
                while let Ok(pos) = req_recv.recv() {
                    tile_send.send((pos, Self::gen_tile(&pos))).unwrap();
                }
            });
        }

        TileBuilder {
            cache: BTreeMap::new(),
            sender: req_send,
            receiver: tile_recv,
        }
    }

    pub fn gen_tile(p: &TilePos) -> Image {
        // the sin() and log2() can be optimized
        let size = 256;
        let mut data = Vec::with_capacity(size as usize * size as usize * 4);

        let pos = p.square();

        let min = pos.corner_min();
        let max = pos.corner_max();

        for y in 0..size {
            for x in 0..size {
                let border = (y == 0 || y == size  - 1) || (x == 0 || x == size-1);

                let px = (x as f64 + 0.5) / size as f64;
                let py = (y as f64 + 0.5) / size as f64;

                let x = min.x as f64 *(1.0 - px) + max.x as f64 * px;
                let y = min.y as f64 *(1.0 - py) + max.y as f64 * py;

                let pi3 = std::f64::consts::FRAC_PI_3;
                let t = (x*x + y*y).sqrt()*5.0;


                let c = V2::new(x, y);
                let mut z = V2::zero();
                let mut t = 0.0;

                let c2 = c.x*c.x + c.y*c.y;

                // skip computation inside M1 - http://iquilezles.org/www/articles/mset_1bulb/mset1bulb.htm
                let in_m1 = 256.0*c2*c2 - 96.0*c2 + 32.0*c.x - 3.0 < 0.0;

                // skip computation inside M2 - http://iquilezles.org/www/articles/mset_2bulb/mset2bulb.htm
                let in_m2 = 16.0*(c2+2.0*c.x+1.0) - 1.0 < 0.0;

                if border || in_m1 || in_m2 {
                    t = 255.0;
                } else {
                    for i in 0..256 {
                        z = V2::new(
                            z.x*z.x - z.y*z.y,
                            2.0*z.x*z.y
                        ) + c;

                        let d = z.x*z.x + z.y*z.y;
                        if d > 256.0 {
                            t += -d.log2().log2() + 4.0;
                            break;
                        }
                        t += 1.0;
                    }
                }

                let a = (1.0 - (t/(256.0)).powi(2)).min(1.0).max(0.0);
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

        Image { size: V2::new(size, size), data }
    }

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
                let _ = self.sender.try_send(*p);
                todo_count += 1;
            }
        }

        println!("todo: {}, done: {}", todo_count, done_count);

        self.cache = new_cache;
        &self.cache
    }
}

