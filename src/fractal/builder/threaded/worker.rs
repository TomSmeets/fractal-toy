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
    loop {
        match h.recv() {
            Err(_) => break,
            Ok(None) => h.wait(),
            Ok(Some(next)) => {
                let t = TileContent::new(super::super::cpu::build(&next));
                use crate::fractal::queue::TileResponse;
                if let Err(_) = h.send(TileResponse {
                    pos: next.pos,
                    version: next.version,
                    content: t,
                }) {
                    break;
                }
            },
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.handle.take().unwrap().join().unwrap();
    }
}
