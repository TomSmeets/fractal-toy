use crate::fractal::builder::TileParams;
use crate::fractal::builder::TileRequest;
use crate::fractal::Task;
use crate::fractal::TaskMap;
use crate::fractal::TileContent;
use crate::fractal::Viewport;
use crossbeam_channel::bounded;
use crossbeam_channel::{Receiver, Sender};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, MutexGuard};
use tilemap::TilePos;

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
        loop {
            if self.master_lock.load(Ordering::Relaxed) {
                // wait a bit
                std::thread::sleep(std::time::Duration::from_millis(20));
                std::thread::yield_now();
                continue;
            }

            return self.m.lock().unwrap();
        }
    }

    pub fn lock_high(&self) -> MutexGuard<T> {
        self.master_lock.store(true, Ordering::Relaxed);
        let l = self.m.lock().unwrap();
        self.master_lock.store(false, Ordering::Relaxed);
        l
    }
}

pub struct TaskMapWithParams {
    pub quit: bool,
    pub map: TaskMap,
    pub params: TileParams,
    pub params_version: usize,
}

pub struct TileResponse {
    pub pos: TilePos,
    pub version: usize,
    pub content: TileContent,
}

pub struct Queue {
    pub tiles: Arc<PrioMutex<TaskMapWithParams>>,
    rx: Receiver<TileResponse>,

    // NOTE: clone to create handles, could be done differently
    handle: QueueHandle,
}

#[derive(Clone)]
pub struct QueueHandle {
    tx: Sender<TileResponse>,
    pub tiles: Arc<PrioMutex<TaskMapWithParams>>,
}

impl Queue {
    pub fn set_params(&mut self, p: &TileParams) {
        // update params
        let mut m = self.tiles.lock_high();
        m.params = p.clone();
        m.params_version = m.params_version.wrapping_add(1);
        println!("set params {}", m.params_version);

        // clear map
        m.map.clear();
    }

    pub fn update(&mut self, vp: &Viewport) -> usize {
        // update params
        let mut m = self.tiles.lock_high();

        let new_iter = vp.get_pos_all();
        m.map.update_with(new_iter, |_, _| (), |_| Some(Task::Todo));

        m.params_version
    }

    pub fn new(params: TileParams) -> Queue {
        // This channel contains newly finished tiles
        // These get cleard each frame wich is around 60 times per second
        // The exact size does not matter much
        let (out_tx, out_rx) = bounded(64);

        let tiles = Arc::new(PrioMutex::new(TaskMapWithParams {
            quit: false,
            map: TaskMap::new(),
            params_version: 0,
            params,
        }));

        Queue {
            tiles: tiles.clone(),
            rx: out_rx,
            handle: QueueHandle { tx: out_tx, tiles },
        }
    }

    pub fn handle(&self) -> QueueHandle {
        self.handle.clone()
    }

    pub fn try_recv(&self) -> Result<TileResponse, ()> {
        self.rx.try_recv().map_err(|_| ())
    }
}

impl Drop for Queue {
    fn drop(&mut self) {
        self.tiles.lock_high().quit = true;
    }
}

impl QueueHandle {
    pub fn wait(&self) {
        // TODO: thread parking? what is the performance? (we will have to do this every frame)
        std::thread::sleep(std::time::Duration::from_millis(20));
        std::thread::yield_now();
    }

    pub fn recv(&self) -> Result<Option<TileRequest>, ()> {
        let mut ts = self.tiles.lock();

        if ts.quit {
            return Err(());
        }

        let pos = ts
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

        let pos = match pos {
            Some(n) => n,
            None => return Ok(None),
        };

        Ok(Some(TileRequest {
            // TODO: don't clone params, just return pos
            params: ts.params.clone(),
            version: ts.params_version,
            pos,
        }))
    }

    pub fn send(&self, t: TileResponse) -> Result<(), ()> {
        self.tx.send(t).map_err(|_| ())
    }
}
