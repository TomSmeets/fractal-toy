use sdl2::event::*;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;

use crate::math::*;

#[derive(Debug, Clone, Copy)]
pub enum InputAction {
    None,

    Quit,

    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,

    LookUp,
    LookDown,
    LookLeft,
    LookRight,

    A,
    B,
    X,
    Y,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,

    Count,
}

pub struct Input {
    pub mouse: V2i,
    pub mouse_down: bool,

    pub scroll: i32,
    pub dir_move: V2,
    pub dir_look: V2,

    action: [bool; (InputAction::Count as usize)],
}

impl Default for Input {
    fn default() -> Self {
        Input::new()
    }
}

fn limit(v: V2) -> V2 {
    let mag = v.magnitude2();

    if mag > 1.0 {
        v * (1.0 / mag.sqrt())
    } else {
        v
    }
}

impl Input {
    pub fn new() -> Self {
        Input {
            mouse: V2i::zero(),
            mouse_down: false,
            scroll: 0,
            dir_move: V2::zero(),
            dir_look: V2::zero(),
            action: [false; InputAction::Count as usize],
        }
    }

    pub fn begin(&mut self) {
        self.scroll = 0;
        self.dir_look = V2::zero();
        self.dir_move = V2::zero();
    }

    pub fn end(&mut self) {
        self.dir_look = limit(self.dir_look);
        self.dir_move = limit(self.dir_move);
    }

    pub fn action(&self, act: InputAction) -> bool {
        self.action[act as usize]
    }

    pub fn handle_sdl(&mut self, events: &[Event]) {
        for e in events {
            match e {
                Event::Quit { .. } => self.action[InputAction::Quit as usize] = true,
                Event::KeyUp {
                    keycode: Some(key), ..
                } => self.handle_sdl_key(*key, false),

                Event::KeyDown {
                    keycode: Some(key), ..
                } => self.handle_sdl_key(*key, true),

                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Left,
                    ..
                } => self.mouse_down = true,
                Event::MouseButtonUp {
                    mouse_btn: MouseButton::Left,
                    ..
                } => self.mouse_down = false,

                Event::MouseWheel { y, .. } => self.scroll += y,
                Event::MouseMotion { x, y, .. } => {
                    self.mouse.x = *x as i32;
                    self.mouse.y = *y as i32;
                },
                _ => (),
            }
        }

        if self.action(InputAction::MoveUp) {
            self.dir_move.y += 1.;
        }
        if self.action(InputAction::MoveDown) {
            self.dir_move.y -= 1.;
        }
        if self.action(InputAction::MoveRight) {
            self.dir_move.x += 1.;
        }
        if self.action(InputAction::MoveLeft) {
            self.dir_move.x -= 1.;
        }

        if self.action(InputAction::LookUp) {
            self.dir_look.y += 1.;
        }
        if self.action(InputAction::LookDown) {
            self.dir_look.y -= 1.;
        }
        if self.action(InputAction::LookRight) {
            self.dir_look.x += 1.;
        }
        if self.action(InputAction::LookLeft) {
            self.dir_look.x -= 1.;
        }
    }

    fn sdl_key_to_action(&self, key: Keycode) -> InputAction {
        match key {
            Keycode::W => InputAction::MoveUp,
            Keycode::S => InputAction::MoveDown,
            Keycode::D => InputAction::MoveRight,
            Keycode::A => InputAction::MoveLeft,

            Keycode::I => InputAction::LookUp,
            Keycode::K => InputAction::LookDown,
            Keycode::L => InputAction::LookRight,
            Keycode::J => InputAction::LookLeft,

            Keycode::Q => InputAction::B,
            Keycode::E => InputAction::A,
            Keycode::R => InputAction::X,
            Keycode::F => InputAction::Y,

            Keycode::Num1 => InputAction::F1,
            Keycode::Num2 => InputAction::F2,
            Keycode::Num3 => InputAction::F3,
            Keycode::Num4 => InputAction::F4,
            Keycode::Num5 => InputAction::F5,
            Keycode::Num6 => InputAction::F6,
            Keycode::Num7 => InputAction::F7,
            Keycode::Num8 => InputAction::F8,
            Keycode::Num9 => InputAction::F9,
            Keycode::Num0 => InputAction::F10,
            Keycode::Minus => InputAction::F11,
            Keycode::Equals => InputAction::F12,

            _ => InputAction::None,
        }
    }

    fn handle_sdl_key(&mut self, key: Keycode, down: bool) {
        // println!("key {:?} {}", key, if down { "down" } else { "up" });
        let act = self.sdl_key_to_action(key);
        let was_down = self.action(act);
        if !was_down && down {
            eprintln!("action: {:?} = {}", act, if down { "down" } else { "up" });
        }

        self.action[act as usize] = down;
    }
}
