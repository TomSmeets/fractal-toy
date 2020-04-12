use super::Config;
use crate::core::Maze;
use crate::core::MazeBuilder;
use crate::core::Tile;
use std::io::Write;
use std::io::{stdin, stdout};
use std::thread;
use std::time::Duration;
use termion::event::{Event, Key};
use termion::input::{MouseTerminal, TermRead};
use termion::raw::*;
use termion::screen::AlternateScreen;
use termion::*;

pub fn run(cfg: Config) {
    let mut gen = MazeBuilder::new(cfg.width, cfg.height);
    let stdout = stdout().into_raw_mode().unwrap();
    let stdout = MouseTerminal::from(stdout);
    let mut stdout = AlternateScreen::from(stdout);
    let stdin = stdin();

    let out = &mut stdout;
    let gen = &mut gen;
    write!(out, "{}", clear::All).unwrap();
    while let Some(_) = gen.next() {
        write!(out, "{}", cursor::Goto(1, 1)).unwrap();
        write!(out, "Generating...").unwrap();
        write!(out, "{}", cursor::Goto(1, 2)).unwrap();
        out.flush().unwrap();
        show(&gen.maze, out);
        write!(out, "{}", color::Bg(color::Blue)).unwrap();
        for ((x, y), _) in gen.queue.iter() {
            write!(
                out,
                "{}  ",
                cursor::Goto(((*x) * 2) as u16 + 1, *y as u16 + 2)
            )
            .unwrap();
        }

        write!(out, "{}", style::Reset).unwrap();
        write!(out, "{}", cursor::Goto(1, 1)).unwrap();

        out.flush().unwrap();
        thread::sleep(Duration::from_millis(cfg.delay));
    }

    write!(out, "Done! press 'q' to quit.").unwrap();
    out.flush().unwrap();

    for c in stdin.events() {
        match c.unwrap() {
            Event::Key(Key::Char('q')) => break,
            _ => {}
        };
        stdout.flush().unwrap();
    }
}

fn show(maze: &Maze, out: &mut impl Write) {
    for y in 0..maze.data.size_y {
        write!(out, "{}", cursor::Goto(1, y as u16 + 2)).unwrap();
        for x in 0..maze.data.size_x {
            match maze.at((x, y)) {
                Tile::Undefined => write!(out, "??").unwrap(),
                Tile::Empty => write!(out, "  ").unwrap(),
                Tile::Wall => write!(out, "{}  {}", style::Invert, style::Reset).unwrap(),
            }
        }
    }
}
