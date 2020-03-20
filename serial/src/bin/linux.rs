use serial::game::State;
use serial::module::input::InputAction;

fn main() {
    let mut s = State::new();
    loop {
        if s.update() {
            break;
        }

        let save = s.input.is_down(InputAction::F5);
        let load = s.input.is_down(InputAction::F6);

        if save {
            serial::state::save("auto", &s);
        }

        if load {
            drop(s);
            s = serial::state::load("auto").unwrap();
        }
    }
}
