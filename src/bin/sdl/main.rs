mod atlas;
mod game;
mod sdl;

use self::game::State;
use fractal_toy::state::{load, save};

fn main() {
    let mut s = State::new();

    if let Ok(x) = load("auto") {
        s.load(x);
    }

    loop {
        if s.update() {
            break;
        }

        let do_save = s.input.save;
        let do_load = s.input.load;

        if do_save {
            save("manual", &s.save());
        }

        if do_load {
            match load("manual") {
                Ok(x) => s.load(x),
                Err(e) => eprintln!("{}", e),
            };
        }
    }

    fractal_toy::state::save("auto", &s.save());
}
