use super::TilePos;
use crate::compare_iter::{CompareIter, ComparedValue};
use std::collections::BTreeMap;

type Iter<'a, T> = std::collections::btree_map::Iter<'a, TilePos, T>;
type IterMut<'a, T> = std::collections::btree_map::IterMut<'a, TilePos, T>;

// TODO: move task out of here
#[derive(Debug)]
pub enum Task<T> {
    Todo,
    Doing,
    Done(T),
}

/// Remembers generated tiles, and adds new ones
pub struct TileMap<T> {
    // TODO: index with (Version, TilePos)
    pub tiles: BTreeMap<TilePos, T>,
}

impl<T> TileMap<T> {
    pub fn new() -> Self {
        TileMap {
            tiles: BTreeMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.tiles.clear();
    }

    pub fn iter(&self) -> Iter<T> {
        self.tiles.iter()
    }


    pub fn iter_mut(&mut self) -> IterMut<T> {
        self.tiles.iter_mut()
    }

    pub fn get_mut(&mut self, k: &TilePos) -> Option<&mut T> {
        self.tiles.get_mut(k)
    }

    // No more destructor please, use drop
    pub fn update_with<I, FDel, FNew>(&mut self, new_iter: I, mut delete: FDel, mut insert: FNew)
    where
        I: Iterator<Item = TilePos>,
        FDel: FnMut(TilePos, T),
        FNew: FnMut(TilePos) -> Option<T>,
    {
        // items we rendered last frame
        let tiles = std::mem::replace(&mut self.tiles, BTreeMap::new());
        let old_iter = tiles.into_iter();

        // items we should render this frame

        let iter = CompareIter::new(old_iter, new_iter, |l, r| l.0.cmp(r));
        for i in iter {
            match i {
                ComparedValue::Left((k, v)) => {
                    // only in old_iter, remove value
                    delete(k, v);
                },
                ComparedValue::Right(r) => {
                    if let Some(v) =  insert(r) {
                        self.tiles.insert(r, v);
                    }
                },
                ComparedValue::Both((k, v), _) => {
                    self.tiles.insert(k, v);
                },
            }
        }
    }
}

impl<T> Default for TileMap<T> {
    fn default() -> Self {
        Self::new()
    }
}
