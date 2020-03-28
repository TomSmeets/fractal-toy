use super::Tile;
use std::io::Write;
use termion::*;

pub struct Maze {
    size_x: i32,
    size_y: i32,
    data: Vec<Tile>,
}

impl Maze {
    pub fn new(sx: u32, sy: u32) -> Maze {
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

    pub fn show(&self, out: &mut impl Write) {
        for y in 0..self.size_y {
            for x in 0..self.size_x {
                match self.at((x, y)) {
                    Tile::Undefined => write!(out, "??").unwrap(),
                    Tile::Empty => write!(out, "  ").unwrap(),
                    Tile::Wall => write!(out, "{}  {}", style::Invert, style::Reset).unwrap(),
                }
            }
            writeln!(out, "{}", cursor::Goto(1, y as u16 + 1)).unwrap();
        }
    }
}
