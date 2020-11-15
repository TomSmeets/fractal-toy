mod atlas;
mod game;
mod input;
mod main2;
mod sdl;

use self::game::State;
use fractal_toy::math::Rect;
use fractal_toy::Persist;
use fractal_toy::Reload;

pub fn rect_to_sdl(r: Rect) -> sdl2::rect::Rect {
    sdl2::rect::Rect::new(r.pos.x, r.pos.y, r.size.x as u32, r.size.y as u32)
}

fn main() {
    main2::run();

    return;

    let p = Persist::new();
    let mut s = State::new();

    if let Ok(x) = p.load("auto") {
        s.load(x);
    }

    loop {
        if s.update() {
            break;
        }

        if s.input.input.save {
            s.input.input.save = false;
            p.save("manual", &s.save()).unwrap();
        }

        if s.input.input.load {
            match p.load("manual") {
                Ok(x) => s.load(x),
                Err(e) => eprintln!("{}", e),
            };
        }
    }

    p.save("auto", &s.save()).unwrap();
}
