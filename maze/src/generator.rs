use rand::prelude::*;
use rand::Rng;
use std::io::{stdin, stdout, Write};
use termion::raw::*;
use termion::*;

use crate::maze::Maze;
use crate::tile::Tile;

pub struct Generator {
    pub queue: Vec<(i32, i32)>,
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
