use super::{
    gen::Gen,
    tile::{TileContent, TilePos},
};
use std::{
    sync::{Arc, Mutex, RwLock},
    thread,
};

pub type TileQueue = WorkQueue<TilePos, TileContent>;

// TODO: it should be possible to generate wihout thread support
pub struct Worker {
    quit: Arc<RwLock<bool>>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(gen: &mut Arc<RwLock<Gen>>, map: &mut Arc<Mutex<TileQueue>>) -> Worker {
        let quit = Arc::new(RwLock::new(false));

        let handle = {
            let map = Arc::clone(&map);
            let gen = Arc::clone(&gen);
            let quit = Arc::clone(&quit);
            thread::spawn(move || worker(gen, map, quit))
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

fn worker(gen: Arc<RwLock<Gen>>, q: Arc<Mutex<TileQueue>>, quit: Arc<RwLock<bool>>) {
    loop {
        if *quit.read().unwrap() {
            break;
        }

        let next: Option<TilePos> = q.lock().unwrap().pop_todo();
        match next {
            Some(p) => {
                let g = gen.read().unwrap();
                let mut t = TileContent::new();
                t.generate(&g, p);
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

// TODO: could be represented  as a single vec like this
//  [done..,doing..,todo..]
pub struct WorkQueue<K, V> {
    pub todo: Vec<K>,
    pub doing: Vec<K>,
    pub done: Vec<(K, V)>,
}

impl<K, V> WorkQueue<K, V> {
    pub fn new() -> Self {
        WorkQueue {
            todo: Vec::new(),
            doing: Vec::new(),
            done: Vec::new(),
        }
    }
}

impl<K: Copy + Eq, V> WorkQueue<K, V> {
    pub fn push_done(&mut self, k: K, v: V) {
        self.done.push((k, v));
        let idx = self.doing.iter().position(|x| *x == k).unwrap();
        self.doing.swap_remove(idx);
    }

    pub fn drain_done(&mut self) -> Vec<(K, V)> {
        std::mem::replace(&mut self.done, Vec::new())
    }

    pub fn pop_todo(&mut self) -> Option<K> {
        let k = self.todo.pop()?;
        self.doing.push(k);
        Some(k)
    }
}

impl<K, V> Default for WorkQueue<K, V> {
    fn default() -> Self {
        WorkQueue::new()
    }
}
