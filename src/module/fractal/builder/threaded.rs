use super::queue::*;
use std::sync::{Arc, Mutex};
mod worker;
use self::worker::Worker;

pub struct ThreadedTileBuilder {
    pub workers: Vec<Worker>,
}

impl ThreadedTileBuilder {
    pub fn new(queue: Arc<Mutex<TileQueue>>) -> Self {
        #[cfg(feature = "platform-sdl")]
        let n = (sdl2::cpuinfo::cpu_count() - 1).max(1);

        #[cfg(not(feature = "platform-sdl"))]
        let n = 4;

        let mut workers = Vec::with_capacity(n as usize);
        println!("spawning {} workers", n);
        for _ in 0..n {
            workers.push(Worker::new(Arc::clone(&queue)));
        }
        Self { workers }
    }
}

impl Drop for ThreadedTileBuilder {
    fn drop(&mut self) {
        for w in self.workers.iter_mut() {
            w.quit();
        }
    }
}
