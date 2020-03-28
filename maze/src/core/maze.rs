use super::Tile;

pub struct Maze {
    pub size_x: i32,
    pub size_y: i32,
    pub data: Vec<Tile>,
}

impl Maze {
    pub fn new(sx: u32, sy: u32) -> Maze {
        let sx = sx * 2 + 1;
        let sy = sy * 2 + 1;
        Maze {
            size_x: sx as i32,
            size_y: sy as i32,
            data: vec![Tile::Wall; (sx * sy) as usize],
        }
    }

    pub fn at(&self, (x, y): (i32, i32)) -> Tile {
        if x < 0 || y < 0 || x >= self.size_x || y >= self.size_y {
            return Tile::Undefined;
        }
        self.data[(y * self.size_x + x) as usize]
    }

    pub fn set(&mut self, (x, y): (i32, i32), t: Tile) {
        if x < 0 || y < 0 || x >= self.size_x || y >= self.size_y {
            return;
        }
        self.data[(y * self.size_x + x) as usize] = t;
    }
}
