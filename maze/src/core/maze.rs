use super::Array2;
use super::Tile;

pub struct Maze {
    pub data: Array2<Tile>,
}

impl Maze {
    pub fn new(sx: u32, sy: u32) -> Maze {
        let sx = sx * 2 + 1;
        let sy = sy * 2 + 1;
        Maze {
            data: Array2::new(sx, sy, Tile::Wall),
        }
    }

    pub fn size(&self) -> [i32; 2] {
        [self.data.size_x, self.data.size_y]
    }

    pub fn at(&self, p: (i32, i32)) -> Tile {
        self.data.at(p).map(|x| *x).unwrap_or(Tile::Undefined)
    }

    pub fn set(&mut self, p: (i32, i32), t: Tile) {
        self.data.set(p, t)
    }
}
