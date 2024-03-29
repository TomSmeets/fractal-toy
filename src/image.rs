use std::sync::atomic::AtomicU32;
use std::sync::Arc;

use crate::util::*;

// NOTE: clone should be cheap, because we are passing this image around a lot, so we want to clone
// it to prevent borrow checker issues
#[derive(Clone)]
pub struct Image {
    /// Checking if two images are the same would be very expensive if we have to check every pixel
    /// This is why we assign every image a unique id, we only check this for equality
    id: u32,

    size: V2<u32>,

    // TODO: also support images that live in gpu memory?
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
