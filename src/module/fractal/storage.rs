use super::builder::queue::TileQueue;
use super::builder::TileParams;
use super::builder::TileRequest;
use super::viewport::Viewport;
use super::TileTextureProvider;
use crate::iter::compare::{CompareIter, ComparedValue};

/// Remembers generated tiles, and adds new ones
pub struct TileStorage<T> {
    pub tiles: Vec<(TileRequest, T)>,

    /// temporary storage for updating tiles to prevent per frame allocations
    /// should alwyas be empty
    next_frame_tiles: Vec<(TileRequest, T)>,
}

impl<T> Default for TileStorage<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> TileStorage<T> {
    pub fn new() -> Self {
        TileStorage {
            tiles: Vec::new(),
            next_frame_tiles: Vec::new(),
        }
    }

    // TODO: reduce argument count
    pub fn update_tiles(
        &mut self,
        q: &mut TileQueue,
        params: TileParams,
        pos: &Viewport,
        texture_creator: &mut impl TileTextureProvider<Texture = T>,
    ) {
        // If we have two ordered lists of tile points
        // We can iterate over both lists at the same time and produce three kinds.
        //   drop:    elem(old) && !elem(new)
        //   retain:  elem(old) &&  elem(new)
        //   insert: !elem(old) &&  elem(new)
        //
        // to produce these lists we can do:
        // if old.is_none => insert, new.next();
        // if new.is_none => drop,   old.next();
        // if new.is_none && old.is_none => break;
        // if old < new  => remove, old.next()
        // if old == new => retain, old.next(), new.next()
        // if old > new  => insert, new.next(),
        //
        // oooooo
        // oo......
        // oo......
        // oo......
        //   ......
        //
        // xxxxxx
        // xx....nn
        // xx....nn
        // xx....nn
        //   nnnnnn

        // items we rendered last frame
        let old_iter = self.tiles.drain(..);
        // items we should render this frame
        let new_iter = pos.get_pos_all().map(|pos| TileRequest { pos, params });

        assert!(self.next_frame_tiles.is_empty());

        let iter = CompareIter::new(old_iter, new_iter, |l, r| l.0.cmp(r));

        q.todo.clear();
        for i in iter {
            match i {
                ComparedValue::Left((_, t)) => {
                    // only in old_iter, remove value
                    texture_creator.free(t);
                },
                ComparedValue::Right(r) => {
                    // Only in new_iter: enqueue value
                    // TODO: subtract sorted iters instead of this if
                    if !q.doing.contains(&r) && !q.done.iter().any(|x| x.0 == r) {
                        q.todo.push(r)
                    }
                },
                ComparedValue::Both(l, _) => {
                    // this value should be retained, as it is in new_iter and old_iter
                    self.next_frame_tiles.push(l)
                },
            }
        }
        q.todo.reverse();

        // TODO: add sorted done at beginning when iterating
        // q.done.sort_unstable_by(|(r1, _), (r2, _)| r1.cmp(r2));
        for (k, v) in q.done.drain(..) {
            let tile = texture_creator.alloc(&v.pixels);
            // TODO: what is faster sort or iter?
            self.next_frame_tiles.push((k, tile));
        }

        // This should use timsort and should be pretty fast for this usecase
        // Note that in this spesific case, the normal sort will probably be faster than
        // the unstable sort TODO: profile :)
        self.next_frame_tiles.sort_by(|(r1, _), (r2, _)| r1.cmp(r2));
        std::mem::swap(&mut self.next_frame_tiles, &mut self.tiles);
    }
}
