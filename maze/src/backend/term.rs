use crate::core::Generator;
use crate::core::Maze;
use crate::core::Tile;
use clap::arg_enum;
use std::io::stdout;
use std::io::Write;
use std::thread;
use std::time::Duration;
use structopt::StructOpt;
use termion::raw::*;
use termion::*;

arg_enum! {
    enum TermBackend {
        Fancy,
        Minimal,
    }
}

#[derive(StructOpt)]
struct Config {
    #[structopt(short, long, default_value = "Fancy")]
    #[structopt(possible_values = &TermBackend::variants(), case_insensitive = true)]
    backend: TermBackend,

    #[structopt(short, long, default_value = "26")]
    /// Maze width in number of tiles
    width: u32,

    #[structopt(short, long, default_value = "26")]
    /// Maze width in number of tiles
    height: u32,

    #[structopt(short, long, default_value = "20")]
    /// How long should we wait before drawing the next frame?
    delay: u64,
}

#[allow(dead_code)]
pub fn run() {
    let cfg = Config::from_args();
    match cfg.backend {
        TermBackend::Minimal => super::minimal::run(),
        TermBackend::Fancy => run_fancy(cfg),
    }
}

fn run_fancy(cfg: Config) {
    let mut maze = Maze::new(cfg.width * 2 + 1, cfg.height * 2 + 1);
    let mut gen = Generator::new();
    let mut stdout = stdout().into_raw_mode().unwrap();
    let out = &mut stdout;
    let gen = &mut gen;
    let maze = &mut maze;
    write!(out, "{}", clear::All).unwrap();
    while let Some(_) = gen.next(maze) {
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
        thread::sleep(Duration::from_millis(cfg.delay));
    }
}

fn show(maze: &Maze, out: &mut impl Write) {
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
