/// Like buddy allocation but in 2d
use crate::*;

#[derive(Clone, Eq, PartialEq)]
struct Block {
    pos: V2<i32>,
    size: i32,
}

impl Block {
    #[rustfmt::skip]
    pub fn split(self) -> [Block; 4] {
        let size = self.size / 2;

        [
            Block { pos: self.pos + V2::new(0,    0),    size, },
            Block { pos: self.pos + V2::new(size, 0),    size, },
            Block { pos: self.pos + V2::new(0,    size), size, },
            Block { pos: self.pos + V2::new(size, size), size, },
        ]
    }

    pub fn parent(&self) -> Block {
        let size = self.size * 2;
        let pos = (self.pos / size) * size;
        Block { pos, size }
    }
}

struct Pack {
    size: i32,
    free: Vec<Block>,
}

impl Pack {
    pub fn new(size: i32) -> Self {
        Pack {
            size,
            free: vec![Block {
                pos: V2::zero(),
                size,
            }],
        }
    }

    pub fn alloc(&mut self, size: V2<i32>) -> Option<Block> {
        let size = block_size(size);

        // NOTE: no real need to go in reverse, tbh
        let block = self.free.iter().rposition(|x| x.size >= size)?;
        let mut block = self.free.swap_remove(block);

        // split while it is big
        while block.size > size {
            let [a, b, c, d] = block.split();
            block = a;
            self.free.push(b);
            self.free.push(c);
            self.free.push(d);
        }

        Some(block)
    }

    pub fn free(&mut self, block: Block) {
        // find siblings
        let sibs = self
            .free
            .iter()
            .enumerate()
            .filter(|(_, x)| x.parent() == block.parent())
            .map(|(i, _)| i)
            .collect::<Vec<_>>();

        // if all 3 are already free, then upgrade to the parent block
        if sibs.len() == 3 {
            // iterate in reverse, so that swap remove works
            for s in sibs.into_iter().rev() {
                self.free.swap_remove(s);
            }
            self.free(block.parent());
        } else {
            self.free.push(block);
        }
    }

    pub fn dbg(&self) {
        let pitch = self.size as usize;
        let mut img = vec![' ' as u8; pitch * pitch];

        for b in self.free.iter() {
            for y in (b.pos.y)..(b.pos.y + b.size - 1) {
                for x in (b.pos.x)..(b.pos.x + b.size - 1) {
                    img[y as usize * pitch + x as usize] = '#' as u8;
                }
            }
        }

        let mut result = String::new();
        for l in img.chunks(pitch) {
            for c in l {
                result.push(*c as char);
                result.push(*c as char);
            }
            result.push('\n');
        }

        println!("{}", result);
    }
}

#[test]
pub fn test_it() {
    let mut p = Pack::new(16);
    let a = p.alloc(V2::new(2, 2)).unwrap();
    let b = p.alloc(V2::new(4, 4)).unwrap();
    let c = p.alloc(V2::new(5, 5)).unwrap();
    let d = p.alloc(V2::new(2, 2)).unwrap();
    p.dbg();

    p.free(a);
    p.dbg();

    p.free(b);
    p.dbg();

    p.free(c);
    p.dbg();

    p.free(d);
    p.dbg();
}

fn block_size(size: V2<i32>) -> i32 {
    let mut block_size = 1;
    while block_size < size.x || block_size < size.y {
        block_size *= 2;
    }
    block_size
}