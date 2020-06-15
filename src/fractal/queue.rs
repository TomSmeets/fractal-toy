use crate::fractal::builder::TileParams;
use crate::fractal::builder::TileRequest;
use crate::fractal::TileContent;
use crate::tilemap::TilePos;
use crossbeam_channel::bounded;
use crossbeam_channel::{Receiver, Sender};

pub struct Queue {
    tx: Sender<TileRequest>,
    rx: Receiver<(TileRequest, TileContent)>,
    handle: QueueHandle,
}

#[derive(Clone)]
pub struct QueueHandle {
    tx: Sender<(TileRequest, TileContent)>,
    rx: Receiver<TileRequest>,
}

impl Queue {
    pub fn new() -> Queue {
        // bounds is the amount of tiles that are built within one frame
        // it should be largen enough to saturate the tile builders
        // however, all tiles insed this channel will be built, so making it too big will build
        // tiles that might have left the screen
        // TODO: either predict this boundst
        // TODO: or dynamically change it
        // TODO: or make it small and provide tiles more than once per frame
        let (in_tx, in_rx) = bounded(32);
        let (out_tx, out_rx) = bounded(32);
        let q = Queue {
            tx: in_tx,
            rx: out_rx,
            handle: QueueHandle {
                rx: in_rx,
                tx: out_tx,
            },
        };
        q
    }

    pub fn handle(&self) -> QueueHandle {
        self.handle.clone()
    }

    pub fn try_send(&self, params: TileParams, pos: TilePos) -> Result<(), ()> {
        self.tx
            .try_send(TileRequest { pos, params })
            .map_err(|_| ())
    }

    pub fn try_recv(&self, params: &TileParams) -> Result<Option<(TilePos, TileContent)>, ()> {
        let (r, v) = self.rx.try_recv().map_err(|_| ())?;

        // skip invalid responses
        if &r.params != params {
            return Ok(None);
        }

        Ok(Some((r.pos, v)))
    }
}

impl QueueHandle {
    pub fn recv(&self) -> Result<TileRequest, ()> {
        self.rx.recv().map_err(|_| ())
    }

    pub fn send(&self, rq: TileRequest, px: TileContent) -> Result<(), ()> {
        self.tx.send((rq, px)).map_err(|_| ())
    }
}
