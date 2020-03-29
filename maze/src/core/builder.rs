use super::Maze;
use super::Tile;
use rand::prelude::*;
use rand::rngs::ThreadRng;

pub struct MazeBuilder {
    pub rng: ThreadRng,
    pub maze: Maze,
    pub queue: Vec<(i32, i32)>,
}

impl MazeBuilder {
    pub fn new(sx: u32, sy: u32) -> MazeBuilder {
        MazeBuilder {
            rng: thread_rng(),
            queue: vec![(0, 0)],
            maze: Maze::new(sx, sy),
        }
    }

    pub fn build(mut self) -> Maze {
        while let Some(_) = self.next() {}
        self.maze
    }

    pub fn next(&mut self) -> Option<(i32, i32)> {
        let (x, y) = self.queue.pop()?;
        let mut directions = vec![(0, 1), (1, 0), (-1, 0), (0, -1)];
        directions.shuffle(&mut self.rng);
        self.maze.set((x, y), Tile::Empty);
        for (dx, dy) in directions.into_iter() {
            let p1 = (x + dx, y + dy);
            let p2 = (x + dx * 2, y + dy * 2);
            if self.maze.at(p2) == Tile::Wall {
                self.maze.set(p1, Tile::Empty);
                self.maze.set(p2, Tile::Empty);
                self.queue.push(p2);
            }
        }
        Some((x, y))
    }
}
