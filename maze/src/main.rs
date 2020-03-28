use rand::prelude::*;
use std::io::{stdout, Write};
use std::thread;
use std::time::Duration;
use termion::raw::*;
use termion::*;

mod core;

use crate::core::{Generator, Maze};

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
            write!(
                stdout,
                "{}  ",
                cursor::Goto(((*x) * 2) as u16 + 1, *y as u16 + 1)
            )
            .unwrap();
        }
        write!(stdout, "{}", style::Reset).unwrap();
        write!(stdout, "{}", cursor::Goto(1, 1)).unwrap();
        stdout.flush().unwrap();
        thread::sleep(Duration::from_millis(20));
    }
}
