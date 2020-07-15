use crate::fractal::builder::TileParams;
use crate::fractal::Task;
use crate::fractal::TileContent;
use crate::fractal::Viewport;
use crossbeam_channel::bounded;
use crossbeam_channel::{Receiver, Sender};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use tilemap::TilePos;
use tilemap::TileMap;

/// Mutex that can give the main thread priority over worker threads when locking a mutex
/// NOTE: it does not work that well however, but well leave it in for now
pub struct PrioMutex<T> {
    master_lock: AtomicBool,
    m: Mutex<T>,
}

impl<T> PrioMutex<T> {
    pub fn new(data: T) -> Self {
        PrioMutex {
            master_lock: AtomicBool::new(false),
            m: Mutex::new(data),
        }
    }

    pub fn lock(&self) -> MutexGuard<T> {
        while self.master_lock.load(Ordering::Acquire) {
            // wait a bit
            std::thread::sleep(std::time::Duration::from_millis(20));
            std::thread::yield_now();
        }

        self.m.lock().unwrap()
    }

    pub fn lock_high(&self) -> MutexGuard<T> {
        self.master_lock.store(true, Ordering::Release);
        let l = self.m.lock().unwrap();
        self.master_lock.store(false, Ordering::Release);
        l
    }
}

pub struct TaskMapWithParams {
    pub map: TileMap<Task>,
    pub params: TileParams,
    pub params_version: usize,
}

pub struct TileResponse {
    pub pos: TilePos,
    pub version: usize,
    pub content: TileContent,
}

pub struct Queue {
    pub params_version: usize,
    pub tiles: Arc<PrioMutex<TaskMapWithParams>>,
    rx: Receiver<TileResponse>,

    // NOTE: clone to create handles, could be done differently
    handle: QueueHandle,
}

#[derive(Clone)]
pub struct QueueHandle {
    tx: Sender<TileResponse>,
    pub tiles: Weak<PrioMutex<TaskMapWithParams>>,
}

impl TaskMapWithParams {
    pub fn update(&mut self, vp: &Viewport) {
        let new_iter = vp.get_pos_all();
        self.map
            .update_with(new_iter, |_, _| (), |_| Some(Task::Todo));
    }

    pub fn set_params(&mut self, p: &TileParams) {
        self.params = p.clone();
        self.params_version = self.params_version.wrapping_add(1);
        self.map.clear();
    }

    pub fn recv(&mut self) -> Option<TilePos> {
        let pos = self
            .map
            .iter_mut()
            .filter_map(|(k, v)| match v {
                Task::Todo => {
                    *v = Task::Doing;
                    Some(*k)
                },
                _ => None,
            })
            .next();

        pos
    }
}

impl Queue {
    pub fn set_params(&mut self, p: &TileParams) {
        // update params
        let mut m = self.tiles.lock_high();
        m.set_params(p);
        self.params_version = m.params_version;
        println!("set params {}", m.params_version);
    }

    pub fn update(&mut self, vp: &Viewport) {
        // update params
        if let Ok(mut m) = self.tiles.m.try_lock() {
            m.update(vp);
        }
    }

    pub fn new(params: TileParams) -> Queue {
        // This channel contains newly finished tiles
        // These get cleard each frame wich is around 60 times per second
        // The exact size does not matter much
        let (out_tx, out_rx) = bounded(64);

        let tiles = Arc::new(PrioMutex::new(TaskMapWithParams {
            map: TileMap::new(),
            params_version: 1,
            params,
        }));

        Queue {
            params_version: 1,
            tiles: tiles.clone(),
            rx: out_rx,
            handle: QueueHandle {
                tx: out_tx,
                tiles: Arc::downgrade(&tiles),
            },
        }
    }

    pub fn handle(&self) -> QueueHandle {
        self.handle.clone()
    }

    pub fn try_recv(&self) -> Result<TileResponse, ()> {
        self.rx.try_recv().map_err(|_| ())
    }
}

impl QueueHandle {
    pub fn wait(&self) {
        // TODO: thread parking? what is the performance? (we will have to do this every frame)
        std::thread::sleep(std::time::Duration::from_millis(20));
        std::thread::yield_now();
    }

    pub fn send(&self, t: TileResponse) -> Result<(), ()> {
        self.tx.send(t).map_err(|_| ())
    }
}
