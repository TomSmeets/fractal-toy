use crate::module::fractal::tile::*;

pub type TileQueue = WorkQueue<TilePos, TileContent>;

// TODO: could be represented  as a single vec like this
//  [done..,doing..,todo..]
pub struct WorkQueue<K, V> {
    pub todo: Vec<K>,
    pub doing: Vec<K>,
    pub done: Vec<(K, V)>,
}

impl<K, V> WorkQueue<K, V> {
    pub fn new() -> Self {
        WorkQueue {
            todo: Vec::new(),
            doing: Vec::new(),
            done: Vec::new(),
        }
    }
}

impl<K: Copy + Eq, V> WorkQueue<K, V> {
    pub fn push_done(&mut self, k: K, v: V) {
        self.done.push((k, v));
        let idx = self.doing.iter().position(|x| *x == k).unwrap();
        self.doing.swap_remove(idx);
    }

    pub fn drain_done(&mut self) -> Vec<(K, V)> {
        std::mem::replace(&mut self.done, Vec::new())
    }

    pub fn pop_todo(&mut self) -> Option<K> {
        let k = self.todo.pop()?;
        self.doing.push(k);
        Some(k)
    }
}

impl<K, V> Default for WorkQueue<K, V> {
    fn default() -> Self {
        WorkQueue::new()
    }
}
