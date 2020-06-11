use super::builder::TileParams;
use super::builder::TileRequest;
use super::viewport::Viewport;
use super::TileTextureProvider;
use crate::iter::compare::{CompareIter, ComparedValue};

#[derive(Debug)]
pub enum Task<T> {
    Todo,
    Doing,
    Done(T),
}

/// Remembers generated tiles, and adds new ones
pub struct TileStorage<T> {
    /// These tiles are always sorted with respect to TileRequest
    pub tiles: Vec<(TileRequest, Task<T>)>,

    /// temporary storage for updating tiles to prevent per frame allocations
    /// should alwyas be empty
    next_frame_tiles: Vec<(TileRequest, Task<T>)>,
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

    pub fn insert(&mut self, k: TileRequest, v: Task<T>) {
        self.tiles.push((k, v));

        // This should use timsort and should be pretty fast for this usecase
        // Note that in this spesific case, the normal sort will probably be faster than
        // the unstable sort TODO: profile :)
        self.next_frame_tiles.sort_by(|(r1, _), (r2, _)| r1.cmp(r2));
    }

    // TODO: reduce argument count
    pub fn update_tiles(
        &mut self,
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

        for i in iter {
            match i {
                ComparedValue::Left((_, t)) => {
                    if let Task::Done(t) = t {
                        // only in old_iter, remove value
                        texture_creator.free(t);
                    }
                },
                ComparedValue::Right(r) => {
                    self.next_frame_tiles.push((r, Task::Todo));
                },
                ComparedValue::Both(l, _) => {
                    // this value should be retained, as it is in new_iter and old_iter
                    self.next_frame_tiles.push(l);
                },
            }
        }

        std::mem::swap(&mut self.next_frame_tiles, &mut self.tiles);
    }
}

#[test]
fn test_storage() {
    use super::builder::TileType;
    use super::tile::TileContent;
    use crate::math::Vector2;
    use rand::prelude::*;

    struct Provider {}
    impl TileTextureProvider for Provider {
        type Texture = ();

        fn alloc(&mut self, _: &[u8]) {}

        fn free(&mut self, _: ()) {}
    }

    let mut storage = TileStorage::new();
    let mut provider = Provider {};
    let mut queue = TileQueue::new();
    let params = TileParams {
        // arbitrary params, they do not matter
        kind: TileType::Mandelbrot,
        iterations: 64,
    };
    let viewport = Viewport::new(Vector2::new(800, 600));

    // for now this only test very basic stuff

    // tiles should be empty in the beginning
    assert!(storage.tiles.is_empty());
    // next_frame_tiles should always be empty
    assert!(storage.next_frame_tiles.is_empty());

    storage.update_tiles(&mut queue, params, &viewport, &mut provider);

    // no tiles have been generated, only requested
    assert!(storage.tiles.is_empty());
    assert!(storage.next_frame_tiles.is_empty());

    // all tiles should have been requested and put into queue.todo
    assert!(!queue.todo.is_empty());

    // the todo items should be sorted in reverse
    // NOTE: there is 'is_sorted' but it is not yet stable
    assert!(queue
        .todo
        .iter()
        .zip(queue.todo[1..].iter())
        .all(|(a, b)| a >= b));

    // Pretend that we generated all those tiles
    for r in queue.todo.drain(..) {
        queue.done.push((r, TileContent::new(Vec::new())));
    }

    let mut rng = thread_rng();
    // queue.done can be orderd in any way
    // it should not matter
    queue.done.shuffle(&mut rng);

    storage.update_tiles(&mut queue, params, &viewport, &mut provider);
    // Now, however, there should be tiles stored, because we generated them
    assert!(!storage.tiles.is_empty());
    // the tiles should be sorted
    // NOTE: there is 'is_sorted' but it is not yet stable
    assert!(storage
        .tiles
        .iter()
        .zip(storage.tiles[1..].iter())
        .all(|(a, b)| a <= b));

    // and we generated all tiles, so none should be enqueued
    assert!(queue.todo.is_empty());
    // and all done tiles are removed from the queue
    assert!(queue.done.is_empty());
}
