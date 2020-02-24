#[derive(Debug)]
pub struct QuadTree<T> {
    value: Option<T>,
    nodes: Option<Box<[Self; 4]>>,
}


#[derive(Debug)]
#[derive(PartialEq, Eq)]
#[derive(Clone, Copy)]
pub struct QuadTreePosition {
    x : u32,
    y : u32,
    z : u32,
}

impl<T> QuadTree<T> {
    fn new() -> Self {
        QuadTree {
            value : None,
            nodes : None,
        }
    }

    fn reduce_to(&mut self, level: i32) {
        if level == 0 {
            self.nodes = None;
            return
        }

        if let Some(ns) = &mut self.nodes {
            for n in ns.as_mut() {
                n.reduce_to(level - 1);
            }
        }
    }


    fn insert_value(&mut self, value: T) { self.value = Some(value); }
    fn remove_value(&mut self)           { self.value = None; }
    fn reduce_to_self(&mut self) { self.nodes = None; }


    fn insert_at(&mut self, pos: QuadTreePosition, value: T) {
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
        let qx = pos.x >> shift_count;
        let qy = pos.y >> shift_count;
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


    fn get_at(&mut self, pos: QuadTreePosition) -> Option<&mut T>{
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
        let qx = pos.x >> shift_count;
        let qy = pos.y >> shift_count;
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
    fn values(&self) -> Vec<&T> {
        let mut vs = Vec::new();

        match &self.value {
            None    => {},
            Some(v) => vs.push(v),
        }

        match &self.nodes {
            None     => { },
            Some(ns) => for n in ns.as_ref() {
                vs.append(&mut n.values());
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
