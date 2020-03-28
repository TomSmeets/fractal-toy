/// `backend` is not a good name. This module is more like a frontend.
/// All backends shuld provide is platform spesific stuff. Config might even be moved into core. Im not shure about that yet.
/// TODO: graphical backends: sdl2, opengl, vulkan
use clap::arg_enum;
use structopt::StructOpt;

pub mod minimal;
pub mod term;

arg_enum! {
    pub enum TermBackend {
        Fancy,
        Minimal,
    }
}

#[derive(StructOpt)]
pub struct Config {
    #[structopt(short, long, default_value = "Fancy")]
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
        TermBackend::Fancy => self::term::run(cfg),
    }
}