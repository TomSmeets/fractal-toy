use std::io::stdout;
use termion::raw::*;

mod backend;
mod core;

use crate::backend::term as back;
use crate::core::{Generator, Maze};

fn main() {
    let mut m = Maze::new(53, 53);
    let mut gen = Generator::new();

    let mut stdout = stdout().into_raw_mode().unwrap();
    back::init(&mut stdout);
    while gen.next(&mut m) {
        back::run(&gen, &m, &mut stdout);
    }
}
