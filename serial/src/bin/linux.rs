use serial::game::State;

fn main() {
    let mut s = State::new();
    while !s.update() {}
}
