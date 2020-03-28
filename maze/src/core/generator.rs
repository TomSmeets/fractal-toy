use super::Maze;
use super::Tile;
use rand::prelude::*;
use rand::rngs::ThreadRng;

pub struct Generator {
    pub rng: ThreadRng,
    pub queue: Vec<(i32, i32)>,
}

impl Generator {
    pub fn new() -> Generator {
        Generator {
            rng: thread_rng(),
            queue: vec![(0, 0)],
        }
    }

    pub fn next(&mut self, maze: &mut Maze) -> Option<(i32, i32)> {
        let (x, y) = self.queue.pop()?;

        {
            let mut directions = vec![(0, 1), (1, 0), (-1, 0), (0, -1)];
            directions.shuffle(&mut self.rng);
            maze.set((x, y), Tile::Empty);
            for (dx, dy) in directions.into_iter() {
                let p1 = (x + dx, y + dy);
                let p2 = (x + dx * 2, y + dy * 2);
                if maze.at(p2) == Tile::Wall {
                    maze.set(p1, Tile::Empty);
                    maze.set(p2, Tile::Empty);
                    self.queue.push(p2);
                }
            }
            Some((x, y))
        }
    }
}
