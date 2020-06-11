mod worker;
use self::worker::Worker;
use crate::fractal::builder::TileRequest;
use crate::fractal::tile::TileContent;
use crossbeam_channel::{Receiver, Sender};

pub struct ThreadedTileBuilder {
    pub workers: Vec<Worker>,
}

impl ThreadedTileBuilder {
    pub fn new(rx: Receiver<TileRequest>, tx: Sender<(TileRequest, TileContent)>) -> Self {
        #[cfg(feature = "platform-sdl")]
        let n = (sdl2::cpuinfo::cpu_count() - 1).max(1);

        #[cfg(not(feature = "platform-sdl"))]
        let n = 4;

        let mut workers = Vec::with_capacity(n as usize);
        println!("spawning {} workers", n);
        for _ in 0..n {
            workers.push(Worker::new(rx.clone(), tx.clone()));
        }
        Self { workers }
    }
}