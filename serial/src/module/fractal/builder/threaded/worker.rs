use crate::module::fractal::{
    builder::{queue::*, TileRequest},
    gen::Gen,
    tile::{TileContent, TilePos},
};
use std::{
    sync::{Arc, Mutex, RwLock},
    thread,
};

// TODO: it should be possible to generate wihout thread support
pub struct Worker {
    quit: Arc<RwLock<bool>>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(map: Arc<Mutex<TileQueue>>) -> Worker {
        let quit = Arc::new(RwLock::new(false));

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
        *(self.quit.write().unwrap()) = true;
    }
}

fn worker(q: Arc<Mutex<TileQueue>>, quit: Arc<RwLock<bool>>) {
    loop {
        if *quit.read().unwrap() {
            break;
        }

        let next: Option<TileRequest> = q.lock().unwrap().pop_todo();
        match next {
            Some(p) => {
                let mut t = TileContent {
                    pixels: super::super::cpu::build(p),
                    region: None,
                };
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
