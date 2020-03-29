use super::Config;
use crate::core::Generator;

pub fn run(cfg: Config) {
    let mut gen = Generator::new(cfg.width, cfg.height);
    while let Some(p) = gen.next() {
        println!("gen.next: {:?}", p);
    }
    println!("done");
}
