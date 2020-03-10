use rand::prelude::*;
use rand::Rng;

use termion::*;
use termion::raw::*;

use std::io::{Write, stdout, stdin};

#[derive(Copy, Clone, Eq, PartialEq)]
enum Tile {
    Undefined,
    Empty,
    Wall,
}

struct Maze {
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

struct Generator {
    queue: Vec<(i32, i32)>,
}

impl Generator {
    pub fn new() -> Generator {
        Generator {
            queue: vec![(0, 0)],
        }
    }

    pub fn next(&mut self, maze: &mut Maze, rng: &mut impl Rng) -> bool {
        // self.queue.shuffle(rng);
        if let Some((x, y)) = self.queue.pop() {
            let mut directions = vec![(0, 1), (1, 0), (-1, 0), (0, -1)];
            directions.shuffle(rng);
            maze.set((x, y), Tile::Empty);
            for (dx, dy) in directions.into_iter() {
                let p1 = (x + dx, y + dy);
                let p2 = (x + dx * 2, y + dy * 2);
                if maze.at(p2) == Tile::Wall {
                    maze.set(p1, Tile::Empty);
                    maze.set(p2, Tile::Empty);
                    self.queue.push(p2);
                    // self.generate(rng, p2);
                }
            }
            true
        } else {
            false
        }
    }

}

fn main() {
    let mut m = Maze::new(53, 53);
    let mut rng = thread_rng();
    let mut gen = Generator::new();

    let mut stdout = stdout().into_raw_mode().unwrap();

        write!(stdout, "{}", clear::All).unwrap();
    while gen.next(&mut m, &mut rng) {
        write!(stdout, "{}", cursor::Goto(1, 1)).unwrap();
        m.show(&mut stdout);
        write!(stdout, "{}", color::Bg(color::Blue)).unwrap();
        for (x, y) in gen.queue.iter() {
            write!(stdout, "{}  ", cursor::Goto(((*x) * 2) as u16+1, *y as u16+1)).unwrap();
        }
        write!(stdout, "{}", style::Reset).unwrap();
        write!(stdout, "{}", cursor::Goto(1, 1)).unwrap();
        stdout.flush().unwrap();
        std::thread::sleep_ms(20);
    }
}
