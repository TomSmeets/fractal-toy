use serial::game::State;
use serial::module::input::InputAction;
use serial::state::{load, save};

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
            drop(s);
            s = load("manual").unwrap();
        }
    }
    serial::state::save("auto", &s);
}
