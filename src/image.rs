use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use crate::util::*;

#[derive(Clone)]
pub struct Image {
    id: u32,
    size: V2<u32>,
    data: Arc<Vec<u8>>,
}

// reserve the id 0 to represent nothing
static IMAGE_COUNTER: AtomicU32 = AtomicU32::new(1);

impl Image {
    pub fn new(size: V2<u32>, data: Vec<u8>) -> Self {
        // this will wrap eventually (after running for around 50 days or so)
        // but that should not be a problem. If it is, then just use u64
        let id = IMAGE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Image {
            id,
            size,
            data: Arc::new(data),
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn size(&self) -> V2<u32> {
        self.size
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}
