//! `backend` is not a good name. This module is more like a frontend.
//! All backends shuld provide is platform spesific stuff. Config might even be moved into core. Im not shure about that yet.
//! TODO: graphical backends: sdl2, opengl, vulkan
use clap::arg_enum;
use structopt::StructOpt;

pub mod minimal;

#[cfg(feature = "backend-fancy")]
pub mod term;

pub mod gl;

arg_enum! {
    pub enum TermBackend {
        Fancy,
        Minimal,
        GL,
    }
}

#[derive(StructOpt)]
pub struct Config {
    #[structopt(short, long, default_value = "gl")]
    #[structopt(possible_values = &TermBackend::variants(), case_insensitive = true)]
    backend: TermBackend,

    #[structopt(short, long, default_value = "26")]
    /// Maze width in number of tiles
    width: u32,

    #[structopt(short, long, default_value = "26")]
    /// Maze width in number of tiles
    height: u32,

    #[structopt(short, long, default_value = "20")]
    /// How long should we wait before drawing the next frame?
    delay: u64,
}

#[allow(dead_code)]
pub fn run() {
    let cfg = Config::from_args();
    match cfg.backend {
        TermBackend::Minimal => self::minimal::run(cfg),

        #[cfg(feature = "backend-fancy")]
        TermBackend::Fancy => self::term::run(cfg),

        TermBackend::GL => self::gl::run(cfg),

        b => println!("backend '{}' is unsupported :(", b),
    }
}
