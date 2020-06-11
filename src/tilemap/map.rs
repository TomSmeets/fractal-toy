// TODO: remove all external deps
use crate::fractal::builder::TileParams;
use crate::fractal::builder::TileRequest;
use crate::fractal::viewport::Viewport;
use crate::fractal::TileTextureProvider;
use crate::iter::compare::{CompareIter, ComparedValue};
use std::collections::BTreeMap;

type Iter<'a, T> = std::collections::btree_map::Iter<'a, TileRequest, Task<T>>;
type IterMut<'a, T> = std::collections::btree_map::IterMut<'a, TileRequest, Task<T>>;

// TODO: move task out of here
#[derive(Debug)]
pub enum Task<T> {
    Todo,
    Doing,
    Done(T),
}

/// Remembers generated tiles, and adds new ones
pub struct TileStorage<T> {
    tiles: BTreeMap<TileRequest, Task<T>>,
}

impl<T> TileStorage<T> {
    pub fn new() -> Self {
        TileStorage {
            tiles: BTreeMap::new(),
        }
    }

    pub fn iter(&self) -> Iter<T> {
        self.tiles.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        self.tiles.iter_mut()
    }

    pub fn get_mut(&mut self, k: &TileRequest) -> Option<&mut Task<T>> {
        self.tiles.get_mut(k)
    }

    // TODO: reduce argument count
    pub fn update_tiles(
        &mut self,
        params: TileParams,
        pos: &Viewport,
        texture_creator: &mut impl TileTextureProvider<Texture = T>,
    ) {
        let new_iter = pos.get_pos_all().map(|pos| TileRequest { pos, params });
        self.update_with(new_iter, |_, v| texture_creator.free(v));
    }

    // TODO: reduce argument count
    pub fn update_with<I, FDel>(&mut self, new_iter: I, mut delete: FDel)
    where
        I: Iterator<Item = TileRequest>,
        FDel: FnMut(TileRequest, T),
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
                    if let Task::Done(v) = v {
                        delete(k, v)
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
