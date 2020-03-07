use fractal_art::*;
use structopt::StructOpt;

#[test]
fn test_small_image() {
    let mut cfg = Config::new();
    cfg.seed = Some(0);
    cfg.size = Some((64, 64));
    run(&cfg).unwrap();
}

#[test]
fn test_1x1() {
    let mut cfg = Config::new();
    cfg.seed = Some(0);
    cfg.size = Some((1, 1));
    run(&cfg).unwrap();
}

fn main() -> Result<(), String> {
    let cfg = Config::from_args();
    eprintln!("{:#?}", cfg);
    run(&cfg)
}
