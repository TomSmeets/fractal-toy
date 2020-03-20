pub mod pos;
use self::pos::*;

#[derive(Debug)]
pub struct QuadTree<T> {
    pub value: Option<T>,
    pub nodes: Option<Box<[Self; 4]>>,
}

impl<T> QuadTree<T> {
    pub fn new() -> Self {
        QuadTree {
            value: None,
            nodes: None,
        }
    }

    pub fn reduce_to(&mut self, level: i32) {
        if level == 0 {
            self.reduce_to_self();
            return;
        }

        if let Some(ns) = &mut self.nodes {
            for n in ns.as_mut() {
                n.reduce_to(level - 1);
            }
        }
    }

    pub fn insert_value(&mut self, value: T) {
        self.value = Some(value);
    }

    pub fn remove_value(&mut self) {
        self.value = None;
    }

    pub fn reduce_to_self(&mut self) {
        self.nodes = None;
    }

    pub fn clear(&mut self) {
        self.value = None;
        self.nodes = None;
    }

    pub fn children_or_make(&mut self) -> &mut [Self; 4] {
        self.nodes.get_or_insert_with(|| {
            Box::new([
                QuadTree::new(),
                QuadTree::new(),
                QuadTree::new(),
                QuadTree::new(),
            ])
        })
    }

    pub fn values(&self) -> Vec<(QuadTreePosition, &T)> {
        let mut vs = Vec::new();
        let mut p = QuadTreePosition::root();
        self.values_from(&mut vs, &mut p);
        vs
    }

    fn values_from<'a>(
        &'a self,
        vs: &mut Vec<(QuadTreePosition, &'a T)>,
        p: &mut QuadTreePosition,
    ) {
        if let Some(v) = &self.value {
            vs.push((p.clone(), v));
        }

        if let Some(ns) = &self.nodes {
            for (i, n) in ns.as_ref().iter().enumerate() {
                p.child(i as u8 % 2, i as u8 / 2);
                n.values_from(vs, p);
                p.parent();
            }
        }
    }

    pub fn insert_at(&mut self, p: &[u8], value: T) {
        if p.is_empty() {
            self.value = Some(value);
            return;
        }

        let ns = self.children_or_make();
        ns[p[0] as usize].insert_at(&p[1..], value);
    }

    pub fn at(&mut self, p: &[u8]) -> Option<&mut Self> {
        if p.is_empty() {
            return Some(self);
        }

        let ns = self.children_or_make();
        ns[p[0] as usize].at(&p[1..])
    }

    // pub fn at(&mut self, p: (u8, u8)) -> Option<&mut Self> {
    // self.nodes
    // .map(|ns| &mut (ns.as_mut()[(p.1 * 2 + p.0) as usize]))
    // }
    //
    // pub fn node_at(&mut self, pos: &QuadTreePosition) -> &mut Self {
    // let mut q: &mut Self = &mut self;
    // for (i, j) in pos.path {
    // q = &mut q.children_or_make()[0];
    // }
    // q
    // }
    //
    // pub fn get_at(&mut self, pos: &QuadTreePosition) -> Option<&mut Self> {
    // let mut q: &mut Self = &mut self;
    // for (i, j) in pos.path {
    // match q.nodes {
    // None => return None,
    // Some(ns) => {
    // q = &mut ns[0];
    // }
    // }
    // }
    // Some(q)
    // }
}

// #[test]
// fn test_quad() {
// let mut t = QuadTree::new();
//
// let f = false;
// let t = true;
//
// let mut points = [
// QuadTreePosition::from([(0, 0)]),
// QuadTreePosition { x: 0, y: 0, z: 0 },
// QuadTreePosition { x: 0, y: 0, z: 1 },
// QuadTreePosition { x: 0, y: 0, z: 2 },
// QuadTreePosition { x: 0, y: 0, z: 3 },
// QuadTreePosition { x: 0, y: 0, z: 4 },
// QuadTreePosition { x: 1, y: 1, z: 1 },
// QuadTreePosition { x: 2, y: 2, z: 2 },
// QuadTreePosition { x: 4, y: 4, z: 3 },
// QuadTreePosition { x: 8, y: 8, z: 4 },
// QuadTreePosition { x: 1, y: 0, z: 1 },
// QuadTreePosition { x: 2, y: 1, z: 2 },
// QuadTreePosition { x: 1, y: 3, z: 3 },
// QuadTreePosition { x: 7, y: 6, z: 4 },
// ];
//
// use rand::seq::*;
// use rand::*;
// let mut rng = thread_rng();
// points.shuffle(&mut rng);
// for p in &points {
// t.insert_at(*p, *p);
// }
//
// points.shuffle(&mut rng);
// for p in &points {
// assert_eq!(t.get_at(*p).unwrap(), p);
// }
//
// println!("{:?}", t);
// }
