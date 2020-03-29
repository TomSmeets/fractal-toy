use super::Config;
use crate::core::MazeBuilder;

pub fn run(cfg: Config) {
    let mut gen = MazeBuilder::new(cfg.width, cfg.height);
    while let Some(p) = gen.next() {
        println!("gen.next: {:?}", p);
    }
    println!("done");
}
