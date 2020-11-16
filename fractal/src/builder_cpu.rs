use crate::math::*;
use crate::Config;
use crate::Tile;
use crate::TileMap;
use crate::TileParams;
use crate::TilePos;
use crate::TileType;
use crossbeam_channel::{Receiver, Sender};
use std::thread::JoinHandle;

pub struct BuilderCPU {
    version: u32,
    rx: Receiver<(TilePos, u32, Vec<u8>)>,
    workers: Vec<(Sender<ThreadCommand>, JoinHandle<()>)>,
}

enum ThreadCommand {
    Build(TilePos),
    Configure(TileParams),

    // TODO: either use, or remove
    #[allow(dead_code)]
    Quit,
}

impl BuilderCPU {
    pub fn new() -> Self {
        let (a_tx, a_rx) = crossbeam_channel::bounded(32);
        let mut workers = Vec::new();

        let ncpu = (num_cpus::get() - 1).max(1);
        for _ in 0..ncpu {
            let (q_tx, q_rx) = crossbeam_channel::bounded(8);

            let a_tx = a_tx.clone();

            let thread = std::thread::spawn(move || {
                let mut params = None;
                while let Ok(p) = q_rx.recv() {
                    match p {
                        ThreadCommand::Build(p) => {
                            let params = params.as_ref().unwrap();
                            let px = build(p, params);
                            if a_tx.send((p, params.version, px)).is_err() {
                                break;
                            }
                        },

                        ThreadCommand::Configure(new_params) => {
                            params = Some(new_params);
                        },

                        ThreadCommand::Quit => break,
                    }
                }
            });

            workers.push((q_tx, thread));
        }

        Self {
            version: 0,
            rx: a_rx,
            workers,
        }
    }

    pub fn update(&mut self, config: &Config, map: &mut TileMap) {
        if config.params.version != self.version {
            self.version = config.params.version;

            for (tx, _) in self.workers.iter() {
                tx.send(ThreadCommand::Configure(config.params.clone()))
                    .unwrap();
            }
        }

        let mut done = 0;
        for (pos, version, pixels) in self.rx.try_iter() {
            if version != config.params.version {
                continue;
            }

            if let Some(t) = map.tiles.get_mut(&pos) {
                if let Tile::Doing = *t {
                    done += 1;
                    *t = Tile::Done(pixels);
                }
            }
        }

        let mut queued = 0;
        for (p, t) in map.tiles.iter_mut() {
            if let Tile::Todo = t {
                let mut had_ready_workers = false;

                for (tx, _) in self.workers.iter() {
                    if tx.try_send(ThreadCommand::Build(*p)).is_ok() {
                        *t = Tile::Doing;
                        had_ready_workers = true;
                        queued += 1;
                    }
                }

                if !had_ready_workers {
                    break;
                }
            }
        }

        if done > 0 {
            println!("done: {}", done);
        }

        if queued > 0 {
            println!("queued: {}", queued);
        }

        // TODO: we could use this done count to determine how many tiles we should queue for the
        // next iteration;
    }
}

fn build(pos: TilePos, params: &TileParams) -> Vec<u8> {
    let texture_size = params.size.size as usize;
    let mut pixels = vec![0; texture_size * texture_size * 4];

    match params.kind {
        TileType::Empty => {
            // TODO: move out to a seperate builder, the debug builder
            for y in 0..texture_size {
                for x in 0..texture_size {
                    let i = y * texture_size + x;
                    if x <= 4 || y <= 4 || x >= texture_size - 5 || y >= texture_size - 5 {
                        #[allow(clippy::identity_op)]
                        unsafe {
                            *pixels.get_unchecked_mut(i * 4 + 0) = 64;
                            *pixels.get_unchecked_mut(i * 4 + 1) = 64;
                            *pixels.get_unchecked_mut(i * 4 + 2) = 64;
                            *pixels.get_unchecked_mut(i * 4 + 3) = 255;
                        }
                    } else {
                        let dx = x as i32 * 2 - texture_size as i32;
                        let dy = y as i32 * 2 - texture_size as i32;
                        let r = dx * dx + dy * dy;
                        let l = texture_size as i32;
                        let c = if r < l * l { 255 } else { 0 };
                        #[allow(clippy::identity_op)]
                        unsafe {
                            *pixels.get_unchecked_mut(i * 4 + 0) = c as u8;
                            *pixels.get_unchecked_mut(i * 4 + 1) = (x * c / texture_size) as u8;
                            *pixels.get_unchecked_mut(i * 4 + 2) = (y * c / texture_size) as u8;
                            *pixels.get_unchecked_mut(i * 4 + 3) = 255;
                        }
                    }
                }
            }
        },
        TileType::Mandelbrot => {
            draw_mandel(1.0, pos, params, &mut pixels, |mut z, c| {
                z = cpx_sqr(z) + c;
                z
            });
        },
        TileType::BurningShip => {
            draw_mandel(1.0, pos, params, &mut pixels, |mut z, c| {
                z = cpx_abs(z);
                z = cpx_sqr(z) + c;
                z
            });
        },
        // cube = 1.5, sqr = 1.0
        TileType::ShipHybrid => {
            draw_mandel(2.5, pos, params, &mut pixels, |mut z, c| {
                z = cpx_cube(z) + c; // 1.5
                z = cpx_abs(z);
                z = cpx_sqr(z) + c; // 1.0
                z
            });
        },
    }
    pixels
}

fn draw_mandel<F: Fn(V2, V2) -> V2 + Copy>(
    inc: f64,
    pos: TilePos,
    params: &TileParams,
    pixels: &mut [u8],
    f: F,
) {
    let texture_size = params.size.size as usize;

    let sz_small = params.size.size - 2 * params.size.padding;
    let sz_big = params.size.size;

    let rect = pos.square().scale(sz_big as f64 / sz_small as f64);
    let offset = rect.corner_min();
    let zoom = rect.size();

    let iterations = params.iterations as u32;
    let inv_size = 1.0 / texture_size as f64;
    let inv_iter = 1.0 / iterations as f64;

    for y in 0..texture_size {
        for x in 0..texture_size {
            let mut c0 = Vector2::new(x as f64, y as f64);

            // screen coords 0 - 1
            c0 *= inv_size;
            c0.y = 1.0 - c0.y;

            // -1 , 1
            c0 = zoom * c0 + offset;

            let itr = mandel(inc, iterations, c0, f);

            let mut v = itr * inv_iter;
            v *= v;
            v = 1. - v;

            // let rgb = rq.params.color.get(itr as f64 / 64.0 + 1.0 / 3.0, v);

            let rgb = hsv2rgb(itr as f64 / 64.0, v, v);
            let idx = x + y * texture_size;
            #[allow(clippy::identity_op)]
            unsafe {
                *pixels.get_unchecked_mut(idx * 4 + 0) = rgb[0];
                *pixels.get_unchecked_mut(idx * 4 + 1) = rgb[1];
                *pixels.get_unchecked_mut(idx * 4 + 2) = rgb[2];
                *pixels.get_unchecked_mut(idx * 4 + 3) = 255;
            }
        }
    }
}

fn cpx_mul(a: V2, b: V2) -> V2 {
    V2 {
        x: a.x * b.x - a.y * b.y,
        y: a.x * b.y + a.y * b.x,
    }
}

fn cpx_cube(a: V2) -> V2 {
    cpx_mul(cpx_sqr(a), a)
}

fn cpx_sqr(a: V2) -> V2 {
    V2 {
        x: a.x * a.x - a.y * a.y,
        y: 2.0 * a.x * a.y,
    }
}

fn cpx_abs(a: V2) -> V2 {
    V2 {
        x: a.x.abs(),
        y: -a.y.abs(),
    }
}

// some cool algorithms
// nice: ((|re| + |im|i)^2 + c)^3 + c
fn mandel<F: Fn(V2, V2) -> V2>(inc: f64, max: u32, c: V2, f: F) -> f64 {
    let mut z = V2::zero();
    let mut n = 0.0;
    let max = max as f64;
    loop {
        z = f(z, c);

        if n >= max {
            return max;
        }

        if z.x * z.x + z.y * z.y > 64.0 * 64.0 {
            // mandel
            return n as f64 - (z.x * z.x + z.y * z.y).log2().log2() + 4.0;
        }

        n += inc;
    }
}

fn hsv2rgb(hue: f64, sat: f64, val: f64) -> [u8; 3] {
    let hue = hue.fract();
    let hue = hue * 6.0;
    let part = hue as u32;
    let fract = hue - part as f64;

    // upper limit
    let max = 255.0 * val;
    // lower limit
    let min = 255.0 * val - 255.0 * val * sat;
    // increasing slope
    let inc = fract * max + (1.0 - fract) * min;
    // decreasing slope
    let dec = fract * min + (1.0 - fract) * max;

    // as u8
    let min = min as u8;
    let max = max as u8;
    let inc = inc as u8;
    let dec = dec as u8;
    match part {
        0 => [max, inc, min],
        1 => [dec, max, min],
        2 => [min, max, inc],
        3 => [min, dec, max],
        4 => [inc, min, max],
        5 => [max, min, dec],
        _ => [max, max, max],
    }
}
