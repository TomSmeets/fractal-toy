use gilrs::Event;

enum Event {
    Input(InputEvent),
}

enum InputEvent {
    Gamepad(gilrs::Event)
    MouseMotion(i32, i32)
}
