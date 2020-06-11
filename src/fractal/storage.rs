use super::builder::TileParams;
use super::builder::TileRequest;
use super::viewport::Viewport;
use super::TileTextureProvider;
use crate::iter::compare::{CompareIter, ComparedValue};
use std::collections::BTreeMap;

#[derive(Debug)]
pub enum Task<T> {
    Todo,
    Doing,
    Done(T),
}

/// Remembers generated tiles, and adds new ones
pub struct TileStorage<T> {
    pub tiles: BTreeMap<TileRequest, Task<T>>,
}

impl<T> TileStorage<T> {
    pub fn new() -> Self {
        TileStorage {
            tiles: BTreeMap::new(),
        }
    }

    // TODO: reduce argument count
    pub fn update_tiles(
        &mut self,
        params: TileParams,
        pos: &Viewport,
        texture_creator: &mut impl TileTextureProvider<Texture = T>,
    ) {
        // items we rendered last frame
        let tiles = std::mem::replace(&mut self.tiles, BTreeMap::new());
        let old_iter = tiles.into_iter();

        // items we should render this frame
        let new_iter = pos.get_pos_all().map(|pos| TileRequest { pos, params });

        let iter = CompareIter::new(old_iter, new_iter, |l, r| l.0.cmp(r));
        for i in iter {
            match i {
                ComparedValue::Left((_, v)) => {
                    // only in old_iter, remove value
                    if let Task::Done(v) = v {
                        texture_creator.free(v);
                    }
                },
                ComparedValue::Right(r) => {
                    self.tiles.insert(r, Task::Todo);
                },
                ComparedValue::Both((k, v), _) => {
                    self.tiles.insert(k, v);
                },
            }
        }
    }
}

impl<T> Default for TileStorage<T> {
    fn default() -> Self {
        Self::new()
    }
}
