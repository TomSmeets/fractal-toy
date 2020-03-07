mod color;
mod image;

pub use crate::image::*;
use rand::prelude::*;
use std::path::PathBuf;

pub struct Config {
    pub seed: Option<u64>,
    pub center: Option<(u32, u32)>,
    pub size: Option<(u32, u32)>,
    pub output: Option<PathBuf>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            seed: None,
            center: None,
            size: None,
            output: None,
        }
    }
}
// TODO: make platform independent
fn x11_resolution() -> (u32, u32) {
    let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
    let setup = conn.get_setup();
    let screen = setup.roots().nth(screen_num as usize).unwrap();
    (
        screen.width_in_pixels() as u32,
        screen.height_in_pixels() as u32,
    )
}

pub fn run(cfg: &Config) {
    let mut gen = match cfg.seed {
        Some(s) => SmallRng::seed_from_u64(s),
        None => SmallRng::from_rng(thread_rng()).unwrap(),
    };

    let (w, h) = match cfg.size {
        Some(r) => r,
        None => x11_resolution(),
    };

    println!("resolution: {}x{}", w, h);
    println!("Creating image");
    let mut img = Image::new(w, h);
    println!("generating...");
    img.generate(&mut gen);

    if let Some(path) = &cfg.output {
        println!("Saving to {:?}...", path.display());
        img.save(&path);
    }
}
