#[derive(Default)]
pub struct TileContent {
    pub pixels: Vec<u8>,
}

impl TileContent {
    pub fn new(pixels: Vec<u8>) -> Self {
        TileContent { pixels }
    }
}
