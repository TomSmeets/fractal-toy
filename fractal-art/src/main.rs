use fractal_art::*;
use std::path::PathBuf;

#[test]
fn test_small_image() {
    let mut cfg = Config::new();
    cfg.seed = Some(0);
    cfg.size = Some((64, 64));
    run(&cfg);
}

#[test]
fn test_1x1() {
    let mut cfg = Config::new();
    cfg.seed = Some(0);
    cfg.size = Some((1, 1));
    run(&cfg);
}

fn main() {
    let mut cfg = Config::new();
    cfg.output = Some(PathBuf::from("target/out.bmp"));
    run(&cfg);
}
