use serial::game::State;

fn main() {
    let mut s = State::new();
    loop {
        if s.update() {
            break;
        }
    }
}
