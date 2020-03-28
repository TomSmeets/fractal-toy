mod backend;
mod core;
use crate::backend::minimal as back;

fn main() {
    back::run();
}
