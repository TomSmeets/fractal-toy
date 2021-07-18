use crate::asset_loader::AssetLoader;
use crate::gpu::compute_tile::ComputeTile;
use crate::gpu::GpuDevice;
use crate::tilemap::TilePos;
use crate::util::*;
use crate::image::Image;
use crate::fractal::FractalStep;
use crossbeam_channel::Receiver;
use crossbeam_channel::{bounded, Sender};
use std::collections::BTreeMap;
use std::sync::Arc;

const ITER_COUNT: usize = 1024;

pub struct TileBuilder {
    cache: BTreeMap<TilePos, Option<(Image, u32)>>,

    gpu_sender: Sender<TilePos>,

    sender: Sender<(TilePos, V2)>,
    receiver: Receiver<(TilePos, Image)>,
}

impl TileBuilder {
    pub fn new(
        gpu: Arc<GpuDevice>,
        asset_loader: &mut AssetLoader,
        alg: &[FractalStep],
    ) -> TileBuilder {
        let (req_send, req_recv) = bounded::<(TilePos, V2)>(16);
        let (tile_send, tile_recv) = bounded::<(TilePos, Image)>(16);

        let (req_send_gpu, req_recv_gpu) = bounded::<TilePos>(16);
        {
            let gpu_builder = ComputeTile::load(alg, &gpu, asset_loader);
            let gpu_device = Arc::clone(&gpu);
            let tile_send = tile_send.clone();
            std::thread::spawn(move || {
                while let Ok(pos) = req_recv_gpu.recv() {
                    let img = gpu_builder.build(&gpu_device, &pos);
                    if let Err(_) = tile_send.send((pos, img)) {
                        break;
                    }
                }
            });
        }

        for _ in 0..(num_cpus::get() as i32 - 1).max(1) {
            let tile_send = tile_send.clone();
            let req_recv = req_recv.clone();

            let alg = alg.to_vec();
            std::thread::spawn(move || {
                while let Ok((pos, a)) = req_recv.recv() {
                    let img = Self::gen_tile(&alg, &pos, a);
                    if let Err(_) = tile_send.send((pos, img)) {
                        break;
                    }
                }
            });
        }

        TileBuilder {
            cache: BTreeMap::new(),
            gpu_sender: req_send_gpu,
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

    fn gen_tile(alg: &[FractalStep], p: &TilePos, a: V2) -> Image {
        fn cpx_sqr(z: V2) -> V2 {
            V2 {
                x: z.x * z.x - z.y * z.y,
                y: 2.0 * z.x * z.y,
            }
        }

        fn cpx_cube(z: V2) -> V2 {
            V2 {
                x: z.x * z.x * z.x - 3.0 * z.x * z.y * z.y,
                y: 3.0 * z.x * z.x * z.y - z.y * z.y * z.y,
            }
        }

        fn cpx_abs(z: V2) -> V2 {
            V2 {
                x: z.x.abs(),
                y: z.y.abs(),
            }
        }

        // the sin() and log2() can be optimized
        let size = 256;
        let mut data = vec![0_u8; size as usize * size as usize * 4];

        let pos = p.square();

        let min = pos.corner_min();
        let max = pos.corner_max();

        let center = min * 0.5 + max * 0.5;

        let mut values = Vec::with_capacity(size as usize * size as usize);

        // img -> iter -> type
        // or
        // iter -> type -> img?
        for y in 0..size {
            for x in 0..size {
                let i = (y * size + x) as u32;

                let px = (x as f64 + 0.5) / (size) as f64;
                let py = (y as f64 + 0.5) / (size) as f64;

                let x = min.x as f64 * (1.0 - px) + max.x as f64 * px;
                let y = min.y as f64 * (1.0 - py) + max.y as f64 * py;

                let c: V2<f64> = V2::new(x, y);
                let z: V2<f64> = V2::zero();
                values.push((i, c, z));
            }
        }

        let mut t = 0.0;
        for _ in 0..ITER_COUNT {
            for s in alg.iter() {
                let it = values.iter_mut();
                match s {
                    FractalStep::Conj => {
                        for (_, _, z) in it {
                            z.y = -z.y;
                        }
                    }
                    FractalStep::AbsR => {
                        for (_, _, z) in it {
                            z.x = z.x.abs()
                        }
                    }
                    FractalStep::AbsI => {
                        for (_, _, z) in it {
                            z.y = -z.y.abs()
                        }
                    }
                    FractalStep::Square => {
                        for (_, _, z) in it {
                            *z = cpx_sqr(*z)
                        }
                    }
                    FractalStep::Cube => {
                        for (_, _, z) in it {
                            *z = cpx_cube(*z)
                        }
                    }
                    FractalStep::AddC => {
                        for (_, c, z) in values.iter_mut() {
                            *z = *z + *c;
                        }

                        t += 1.0;
                    }
                }
            }

            for ii in (0..values.len()).rev() {
                let (i, c, z) = unsafe { values.get_unchecked(ii) };
                let i = *i;
                let d = z.x * z.x + z.y * z.y;

                if d > 256.0 {
                    let t = t - d.log2().log2() + 4.0;

                    let pi3 = std::f64::consts::FRAC_PI_3;
                    let a = (1.0 - (t / (1024.0)).powi(2)).min(1.0).max(0.0);
                    let t = t * 0.005;
                    let r = a * ((0.5 - t) * 3.0 * pi3 + pi3 * 0.0).sin();
                    let g = a * ((0.5 - t) * 3.0 * pi3 + pi3 * 1.0).sin();
                    let b = a * ((0.5 - t) * 3.0 * pi3 + pi3 * 2.0).sin();

                    let r = r * r;
                    let g = g * g;
                    let b = b * b;

                    unsafe {
                        *data.get_unchecked_mut(i as usize * 4 + 0) = (r * 255.0) as _;
                        *data.get_unchecked_mut(i as usize * 4 + 1) = (g * 255.0) as _;
                        *data.get_unchecked_mut(i as usize * 4 + 2) = (b * 255.0) as _;
                        *data.get_unchecked_mut(i as usize * 4 + 3) = 255;
                    }
                    values.swap_remove(ii);
                }
            }
        }

        Image::new(V2::new(size, size), data)
    }

    /// Either return a cached tile, or add it to the build queue
    pub fn tile(&mut self, p: &TilePos) -> Option<&Image> {
        let in_cache = self.cache.contains_key(p);

        if !in_cache {
            let result = if p.z < 16 {
                self.gpu_sender.try_send(*p).map_err(|_| ())
            } else {
                self.sender.try_send((*p, V2::zero())).map_err(|_| ())
            };

            // tell a builder to build this tile
            if let Ok(_) = result {
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
                }

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
                }
                None => {
                    new_cache.insert(k, None);
                }
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
