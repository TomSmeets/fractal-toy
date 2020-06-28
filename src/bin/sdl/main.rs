mod atlas;
mod game;
mod sdl;

use self::game::State;
use fractal_toy::state::Persist;

fn main() {
    let p = Persist::new();
    let mut s = State::new();

    if let Ok(x) = p.load("auto") {
        s.load(x);
    }

    loop {
        if s.update() {
            break;
        }

        if s.input.save {
            s.input.save = false;
            p.save("manual", &s.save()).unwrap();
        }

        if s.input.load {
            match p.load("manual") {
                Ok(x) => s.load(x),
                Err(e) => eprintln!("{}", e),
            };
        }
    }

    p.save("auto", &s.save()).unwrap();
}
