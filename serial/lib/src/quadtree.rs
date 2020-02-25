#[derive(Debug)]
pub struct QuadTree<T> {
    value: Option<T>,
    nodes: Option<Box<[Self; 4]>>,
}


#[derive(Debug)]
#[derive(PartialEq, Eq)]
#[derive(Clone, Copy)]
pub struct QuadTreePosition {
    pub x : u64,
    pub y : u64,
    pub z : u64,
}

impl QuadTreePosition {
    fn new(x: u64, y: u64, z: u64) -> Self {
        let q = QuadTreePosition { x, y, z };
        q.check();
        q
    }

    fn check(&self) {
        let dim = self.dim();
        assert!(self.x <= dim);
        assert!(self.y <= dim);
    }

    pub fn dim(&self) -> u64 {
        if self.z == 0 {
            0
        } else {
            1 << (self.z - 1)
        }
    }

    fn root() -> Self {
        Self::new(0, 0, 0)
    }


    fn parent(&self) -> Self {
        QuadTreePosition {
            x: self.x >> 1,
            y: self.y >> 1,
            z: self.z - 1,
        }
    }

    fn child(&self, qx: u8, qy: u8) -> Self {
        QuadTreePosition {
            x: (self.x << 1) + qx as u64,
            y: (self.y << 1) + qy as u64,
            z: self.z + 1,
        }
    }
}

#[test]
fn test_pos() {
    let p = QuadTreePosition::root();

    {
        let q = p.child(0, 0).child(0, 0).child(0, 0);
        assert_eq!(q.z, 3);
        assert_eq!(q.x, 0);
        assert_eq!(q.y, 0);
    }


    {
        let q = p.child(1, 1);
        assert_eq!(q.z, 1);
        assert_eq!(q.x, 1);
        assert_eq!(q.y, 1);
        assert_eq!(q.parent(), p);
    }


    {
        let q = p.child(1, 1).child(1, 1);
        assert_eq!(q.z, 2);
        assert_eq!(q.x, 3);
        assert_eq!(q.y, 3);
        assert_eq!(q.parent().parent(), p);
    }

    {
        let q = p.child(1, 1).child(1, 1).child(1, 1);
        assert_eq!(q.z, 3);
        assert_eq!(q.x, 7);
        assert_eq!(q.y, 7);
        assert_eq!(q.parent().parent().parent(), p);
    }
}



impl<T> QuadTree<T> {
    pub fn new() -> Self {
        QuadTree {
            value : None,
            nodes : None,
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

    pub fn insert_value(&mut self, value: T) { self.value = Some(value); }
    pub fn remove_value(&mut self)           { self.value = None; }
    pub fn reduce_to_self(&mut self) { self.nodes = None; }

    pub fn clear(&mut self) {
        self.value = None;
        self.nodes = None;
    }

    pub fn insert_at(&mut self, pos: QuadTreePosition, value: T) {
        if pos.z == 0 {
            assert_eq!(pos.x, 0);
            assert_eq!(pos.y, 0);
            self.insert_value(value);
            return;
        }

        // the highest bit decides which quadrant we are
        let shift_count = pos.z - 1;
        let highest_bit = 1 << shift_count;

        // shift down the coordinates to extract only the highest bit
        let qx = (pos.x >> shift_count) & 0x1;
        let qy = (pos.y >> shift_count) & 0x1;
        assert!(qx == 0 || qx == 1);
        assert!(qy == 0 || qy == 1);

        let p2 = QuadTreePosition {
            x: pos.x & !highest_bit,
            y: pos.y & !highest_bit,
            z: pos.z - 1,
        };

        // create child nodes if nonexistant
        let ns = self.nodes.get_or_insert_with(||
            Box::new([
                QuadTree::new(),
                QuadTree::new(),
                QuadTree::new(),
                QuadTree::new(),
            ])
        );

        ns[(qy*2 + qx) as usize].insert_at(p2, value);
    }


    pub fn get_at(&mut self, pos: QuadTreePosition) -> Option<&mut T>{
        if pos.z == 0 {
            assert_eq!(pos.x, 0);
            assert_eq!(pos.y, 0);
            return match &mut self.value {
                None    => None,
                Some(v) => Some(v)
            };
        }

        // the highest bit decides which quadrant we are
        let shift_count = pos.z - 1;
        let highest_bit = 1 << shift_count;

        // shift down the coordinates to extract only the highest bit
        let qx = (pos.x >> shift_count) & 0x1;
        let qy = (pos.y >> shift_count) & 0x1;
        assert!(qx == 0 || qx == 1);
        assert!(qy == 0 || qy == 1);

        let p2 = QuadTreePosition {
            x: pos.x & !highest_bit,
            y: pos.y & !highest_bit,
            z: pos.z - 1,
        };

        return match &mut self.nodes {
            None     => None,
            Some(ns) => ns[(qy*2 + qx) as usize].get_at(p2),
        };
    }

    // extract values from top down
    pub fn values(&self) -> Vec<(QuadTreePosition, &T)> {
        self.values_with_pos(QuadTreePosition {
            x: 0,
            y: 0,
            z: 0,
        })
    }

    fn values_with_pos(&self, p: QuadTreePosition) -> Vec<(QuadTreePosition, &T)> {
        let mut vs = Vec::new();

        match &self.value {
            None    => {},
            Some(v) => vs.push((p, v)),
        }

        match &self.nodes {
            None     => { },
            Some(ns) => for (i, n) in ns.as_ref().iter().enumerate() {
                let qx = i % 2;
                let qy = i / 2;

                let p1 = QuadTreePosition {
                    x: (p.x << 1) + qx as u64,
                    y: (p.y << 1) + qy as u64,
                    z: p.z + 1,
                };
                let mut ns = n.values_with_pos(p1);
                vs.append(&mut ns);
            },
        };

        vs
    }
}


#[test]
fn test_quad() {
    let mut t = QuadTree::new();

    let mut points = [
        QuadTreePosition { x: 0, y: 0, z: 0},
        QuadTreePosition { x: 0, y: 0, z: 1},
        QuadTreePosition { x: 0, y: 0, z: 2},
        QuadTreePosition { x: 0, y: 0, z: 3},
        QuadTreePosition { x: 0, y: 0, z: 4},

        QuadTreePosition { x: 1, y: 1, z: 1},
        QuadTreePosition { x: 2, y: 2, z: 2},
        QuadTreePosition { x: 4, y: 4, z: 3},
        QuadTreePosition { x: 8, y: 8, z: 4},

        QuadTreePosition { x: 1, y: 0, z: 1},
        QuadTreePosition { x: 2, y: 1, z: 2},
        QuadTreePosition { x: 1, y: 3, z: 3},
        QuadTreePosition { x: 7, y: 6, z: 4},
    ];

    use rand::*;
    use rand::seq::*;
    let mut rng = thread_rng();
    points.shuffle(&mut rng);
    for p in &points { t.insert_at(*p, *p); }

    points.shuffle(&mut rng);
    for p in &points { assert_eq!(t.get_at(*p).unwrap(), p); }

    println!("{:?}", t);
}
