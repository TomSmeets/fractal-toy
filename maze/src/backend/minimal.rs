use super::Config;
use crate::core::Generator;
use crate::core::Maze;

pub fn run(cfg: Config) {
    let mut maze = Maze::new(cfg.height, cfg.width);
    let mut gen = Generator::new();
    while let Some(p) = gen.next(&mut maze) {
        println!("gen.next: {:?}", p);
    }
    println!("done");
}
