use super::TilePos;
use crate::compare_iter::{CompareIter, ComparedValue};
use std::collections::BTreeMap;

type Iter<'a, T> = std::collections::btree_map::Iter<'a, TilePos, Task<T>>;
type IterMut<'a, T> = std::collections::btree_map::IterMut<'a, TilePos, Task<T>>;

// TODO: move task out of here
#[derive(Debug)]
pub enum Task<T> {
    Todo,
    Doing,
    Done(T),
    Empty(Option<T>),
}

/// Remembers generated tiles, and adds new ones
pub struct TileMap<T> {
    tiles: BTreeMap<TilePos, Task<T>>,
}

impl<T> TileMap<T> {
    pub fn new() -> Self {
        TileMap {
            tiles: BTreeMap::new(),
        }
    }

    pub fn clear(&mut self) {
        for (_, t) in self.tiles.iter_mut() {
            let x = std::mem::replace(t, Task::Empty(None));
            match x {
                Task::Done(v) => *t = Task::Empty(Some(v)),
                _ => (),
            }
        }
    }

    pub fn iter(&self) -> Iter<T> {
        self.tiles.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        self.tiles.iter_mut()
    }

    pub fn get_mut(&mut self, k: &TilePos) -> Option<&mut Task<T>> {
        self.tiles.get_mut(k)
    }

    pub fn update_with<I, FDel>(&mut self, new_iter: I, mut delete: FDel)
    where
        I: Iterator<Item = TilePos>,
        FDel: FnMut(TilePos, T),
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
                    match v {
                        Task::Done(v) => delete(k, v),
                        Task::Empty(Some(v)) => delete(k, v),
                        _ => (),
                    }
                },
                ComparedValue::Right(r) => {
                    self.tiles.insert(r, Task::Todo);
                },
                ComparedValue::Both((k, v), _) => match v {
                    Task::Empty(Some(v)) => delete(k, v),
                    Task::Empty(None) => (),
                    v => {
                        self.tiles.insert(k, v);
                    },
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
