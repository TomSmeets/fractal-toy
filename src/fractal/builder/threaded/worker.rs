use crate::fractal::queue::QueueHandle;
use crate::fractal::TileContent;
use std::thread;

pub struct Worker {
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(h: QueueHandle) -> Self {
        let handle = thread::spawn(move || worker(h));

        Worker {
            handle: Some(handle),
        }
    }
}

fn worker(h: QueueHandle) {
    while let Ok(next) = h.recv() {
        let t = TileContent::new(super::super::cpu::build(&next));
        if let Err(_) = h.send(next, t) {
            break;
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.handle.take().unwrap().join().unwrap();
    }
}
