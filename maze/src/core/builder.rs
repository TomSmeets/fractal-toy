use super::Maze;
use super::Tile;
use rand::prelude::*;
use rand::rngs::ThreadRng;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Direction {
    Left,
    Right,
    Top,
    Bottom,
}

impl Direction {
    pub fn values() -> [Direction; 4] {
        [
            Direction::Left,
            Direction::Right,
            Direction::Top,
            Direction::Bottom,
        ]
    }

    pub fn to_xy(self) -> (i32, i32) {
        match self {
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
            Direction::Bottom => (0, -1),
            Direction::Top => (0, 1),
        }
    }
}

pub struct MazeBuilder {
    pub rng: ThreadRng,
    pub maze: Maze,
    pub queue: Vec<((i32, i32), Direction)>,
}

impl MazeBuilder {
    pub fn new(sx: u32, sy: u32) -> MazeBuilder {
        MazeBuilder {
            rng: thread_rng(),
            queue: vec![((0, 0), Direction::Top)],
            maze: Maze::new(sx, sy),
        }
    }

    pub fn build(mut self) -> Maze {
        while let Some(_) = self.next() {}
        self.maze
    }

    pub fn next(&mut self) -> Option<(i32, i32)> {
        let ((x, y), dir) = self.queue.pop()?;
        let (dx, dy) = dir.to_xy();
        let p1 = (x + dx, y + dy);
        let p2 = (x + dx * 2, y + dy * 2);
        if self.maze.at(p2) == Tile::Wall {
            self.maze.set(p1, Tile::Empty);
            self.maze.set(p2, Tile::Empty);

            let mut directions = Direction::values();
            directions.shuffle(&mut self.rng);
            self.maze.set((x, y), Tile::Empty);
            for &d in directions.iter() {
                self.queue.push((p2, d));
            }
        }
        Some((x, y))
    }
}
