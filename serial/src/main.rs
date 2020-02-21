mod state;
mod game;

use crate::game::*;

fn main() {
    println!("self: {:?}", std::env::current_exe().unwrap());

    let mut s = State::new();

    loop {
        if s.update() { break; }
    }
}
