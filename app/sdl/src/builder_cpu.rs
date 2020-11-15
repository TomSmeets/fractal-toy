use crate::Config;
use crate::Tile;
use crate::TileMap;

use crossbeam_channel::{Receiver, Sender};
use fractal_toy::math::Rect;
use fractal_toy::IsTileBuilder;
use fractal_toy::TileParams;
use fractal_toy::TilePos;
use fractal_toy::TileType;
use fractal_toy::Viewport;
use fractal_toy::TEXTURE_SIZE;
use std::thread::JoinHandle;

pub struct BuilderCPU {
    rx: Receiver<(TilePos, Vec<u8>)>,
    workers: Vec<(Sender<ThreadCommand>, JoinHandle<()>)>,
}

enum ThreadCommand {
    Build(TilePos),
    Configure(TileParams),
    Quit,
}

impl BuilderCPU {
    pub fn new() -> Self {
        let (a_tx, a_rx) = crossbeam_channel::bounded(32);
        let mut workers = Vec::new();

        for _ in 0..6 {
            let (q_tx, q_rx) = crossbeam_channel::bounded(4);

            let a_tx = a_tx.clone();

            let thread = std::thread::spawn(move || {
                let mut b = fractal_toy::CPUBuilder::new();
                while let Ok(p) = q_rx.recv() {
                    match p {
                        ThreadCommand::Build(p) => {
                            let px = b.build(p).pixels;
                            if let Err(_) = a_tx.send((p, px)) {
                                break;
                            }
                        },

                        ThreadCommand::Configure(params) => {
                            b.configure(&params);
                        },

                        ThreadCommand::Quit => {
                            break;
                        },
                    }
                }
            });

            workers.push((q_tx, thread));
        }

        Self { rx: a_rx, workers }
    }

    pub fn update(&mut self, config: &Config, map: &mut TileMap) {
        if config.changed {
            for (tx, _) in self.workers.iter() {
                tx.send(ThreadCommand::Configure(config.params.clone()))
                    .unwrap();
            }
        }

        let mut done = 0;
        for (pos, pixels) in self.rx.try_iter() {
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
                    if let Ok(_) = tx.try_send(ThreadCommand::Build(*p)) {
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
