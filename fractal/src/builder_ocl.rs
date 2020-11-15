use crate::Config;
use crate::TileMap;

pub struct BuilderOCL {}

impl BuilderOCL {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, _config: &Config, _map: &mut TileMap) {}
}
