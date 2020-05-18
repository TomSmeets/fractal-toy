use crate::fractal::builder::{queue::*, TileRequest};
use crate::fractal::tile::TileContent;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Worker {
    quit: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(map: Arc<Mutex<TileQueue>>) -> Worker {
        let quit = Arc::new(AtomicBool::new(false));

        let handle = {
            let quit = Arc::clone(&quit);
            thread::spawn(move || worker(map, quit))
        };

        Worker {
            quit,
            handle: Some(handle),
        }
    }

    pub fn quit(&mut self) {
        self.quit.store(true, Ordering::Relaxed);
    }
}

fn worker(q: Arc<Mutex<TileQueue>>, quit: Arc<AtomicBool>) {
    loop {
        if quit.load(Ordering::Relaxed) {
            break;
        }

        let next: Option<TileRequest> = q.lock().unwrap().pop_todo();
        match next {
            Some(p) => {
                let t = TileContent::new(super::super::cpu::build(p));
                q.lock().unwrap().push_done(p, t);
            },
            None => {
                thread::yield_now();
                // yield will use 100% cpu for some reason, so we also wait a bit
                // TODO: use wait and notify?
                thread::sleep(std::time::Duration::from_millis(50));
            },
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.quit();
        self.handle.take().unwrap().join().unwrap();
    }
}
