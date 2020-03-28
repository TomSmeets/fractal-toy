use rand::prelude::*;
use rand::Rng;
use std::io::{stdin, stdout, Write};
use termion::raw::*;
use termion::*;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Tile {
    Undefined,
    Empty,
    Wall,
}
