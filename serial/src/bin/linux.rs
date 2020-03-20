use serial::game::State;
use serial::module::input::InputAction;
use serial::state::{load, load_in_place, save};

fn main() {
    let mut s = match load("auto") {
        Ok(s) => s,
        Err(_) => State::new(),
    };
    loop {
        if s.update() {
            break;
        }

        let do_save = s.input.is_down(InputAction::F5);
        let do_load = s.input.is_down(InputAction::F6);

        if do_save {
            save("manual", &s);
        }

        if do_load {
            let (s2, err) = load_in_place("manual", s);
            if let Some(e) = err {
                eprintln!("{}", e);
            }
            s = s2;
        }
    }
    serial::state::save("auto", &s);
}
