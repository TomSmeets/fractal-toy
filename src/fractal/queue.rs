use crate::fractal::builder::TileParams;
use crate::fractal::builder::TileRequest;
use crate::fractal::TaskMap;
use crate::fractal::TileContent;
use crossbeam_channel::bounded;
use crossbeam_channel::{Receiver, Sender};
use tilemap::TilePos;

use std::sync::{Arc, Mutex};

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
    pub tiles: Arc<Mutex<TaskMapWithParams>>,
    rx: Receiver<TileResponse>,

    // NOTE: clone to create handles, could be done differently
    handle: QueueHandle,
}

#[derive(Clone)]
pub struct QueueHandle {
    tx: Sender<TileResponse>,
    tiles: Arc<Mutex<TaskMapWithParams>>,
}

use crate::fractal::Task;
use crate::fractal::Viewport;

impl Queue {
    pub fn set_params(&mut self, p: &TileParams) {
        // update params
        let mut m = self.tiles.lock().unwrap();
        m.params = p.clone();
        m.params_version = m.params_version.wrapping_add(1);
        println!("set params {}", m.params_version);

        // clear map
        m.map.clear();
    }

    pub fn update(&mut self, vp: &Viewport) -> usize {
        // update params
        let mut m = self.tiles.lock().unwrap();

        let new_iter = vp.get_pos_all();
        m.map.update_with(new_iter, |_, _| (), |_| Some(Task::Todo));

        m.params_version
    }

    pub fn new() -> Queue {
        // This channel contains newly finished tiles
        // These get cleard each frame wich is around 60 times per second
        // The exact size does not matter much
        let (out_tx, out_rx) = bounded(64);

        let tiles = Arc::new(Mutex::new(TaskMapWithParams {
            quit: false,
            map: TaskMap::new(),
            params_version: 0,
            params: TileParams::default(),
        }));

        let q = Queue {
            tiles: tiles.clone(),
            rx: out_rx,
            handle: QueueHandle { tx: out_tx, tiles },
        };
        q
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
        self.tiles.lock().unwrap().quit = true;
    }
}

impl QueueHandle {
    pub fn wait(&self) {
        // TODO: thread parking? what is the performance? (we will have to do this every frame)
        std::thread::sleep(std::time::Duration::from_millis(20));
        std::thread::yield_now();
    }

    pub fn recv(&self) -> Result<Option<TileRequest>, ()> {
        let mut ts = self.tiles.lock().unwrap();

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
