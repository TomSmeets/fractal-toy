use crate::fractal::builder::TileRequest;
use crate::fractal::tile::TileContent;
use crossbeam_channel::{Receiver, Sender};
use std::thread;

pub struct Worker {
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(rx: Receiver<TileRequest>, tx: Sender<(TileRequest, TileContent)>) -> Self {
        let handle = thread::spawn(move || worker(rx, tx));

        Worker {
            handle: Some(handle),
        }
    }
}

fn worker(rx: Receiver<TileRequest>, tx: Sender<(TileRequest, TileContent)>) {
    while let Ok(next) = rx.recv() {
        let t = TileContent::new(super::super::cpu::build(next));
        if let Err(_) = tx.send((next, t)) {
            break;
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.handle.take().unwrap().join().unwrap();
    }
}
