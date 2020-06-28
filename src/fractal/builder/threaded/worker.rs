use crate::fractal::queue::QueueHandle;
use crate::fractal::TileContent;

pub fn worker(h: QueueHandle) {
    loop {
        match h.recv() {
            Err(_) => break,
            Ok(None) => h.wait(),
            Ok(Some(next)) => {
                let t = TileContent::new(super::super::cpu::build(&next));
                use crate::fractal::queue::TileResponse;
                if h.send(TileResponse {
                    pos: next.pos,
                    version: next.version,
                    content: t,
                })
                .is_err()
                {
                    break;
                }
            },
        }
    }
}
