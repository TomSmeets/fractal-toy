use crate::core::Generator;
use crate::core::Maze;
use crate::core::Tile;
use std::io::Write;
use std::thread;
use std::time::Duration;
use termion::*;

pub fn init(out: &mut impl Write) {
    write!(out, "{}", clear::All).unwrap();
}

pub fn run(gen: &Generator, maze: &Maze, out: &mut impl Write) {
    write!(out, "{}", cursor::Goto(1, 1)).unwrap();
    show(maze, out);
    write!(out, "{}", color::Bg(color::Blue)).unwrap();
    for (x, y) in gen.queue.iter() {
        write!(
            out,
            "{}  ",
            cursor::Goto(((*x) * 2) as u16 + 1, *y as u16 + 1)
        )
        .unwrap();
    }

    write!(out, "{}", style::Reset).unwrap();
    write!(out, "{}", cursor::Goto(1, 1)).unwrap();

    out.flush().unwrap();
    thread::sleep(Duration::from_millis(20));
}

pub fn show(maze: &Maze, out: &mut impl Write) {
    for y in 0..maze.size_y {
        for x in 0..maze.size_x {
            match maze.at((x, y)) {
                Tile::Undefined => write!(out, "??").unwrap(),
                Tile::Empty => write!(out, "  ").unwrap(),
                Tile::Wall => write!(out, "{}  {}", style::Invert, style::Reset).unwrap(),
            }
        }
        writeln!(out, "{}", cursor::Goto(1, y as u16 + 1)).unwrap();
    }
}
