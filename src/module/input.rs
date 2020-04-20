use crate::math::*;
use sdl2::controller::Axis;
use sdl2::{event::*, keyboard::Keycode, mouse::MouseButton};
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Button {
    pub is_down: bool,
    pub was_down: bool,
}

impl Default for Button {
    fn default() -> Self {
        Button::new()
    }
}

impl Button {
    pub fn new() -> Self {
        Button {
            is_down: false,
            was_down: false,
        }
    }

    pub fn went_down(self) -> bool {
        self.is_down && !self.was_down
    }

    pub fn went_up(self) -> bool {
        !self.is_down && self.was_down
    }

    pub fn update(&mut self) {
        self.was_down = self.is_down;
    }
}

#[derive(Serialize, Deserialize)]
pub struct Input {
    pub mouse: V2i,
    pub mouse_down: Button,

    pub scroll: i32,
    pub dir_move: V2,
    pub dir_look: V2,

    action: [Button; InputAction::Count as usize],
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
            mouse_down: Button::new(),
            scroll: 0,
            dir_move: V2::zero(),
            dir_look: V2::zero(),
            action: [Button::new(); InputAction::Count as usize],
        }
    }

    pub fn begin(&mut self) {
        self.scroll = 0;

        for a in self.action.iter_mut() {
            a.update();
        }
    }

    pub fn button(&self, act: InputAction) -> &Button {
        &self.action[act as usize]
    }

    pub fn button_mut(&mut self, act: InputAction) -> &mut Button {
        &mut self.action[act as usize]
    }

    pub fn handle_sdl(&mut self, events: &[Event]) {
        self.mouse_down.update();
        for e in events {
            match e {
                Event::Quit { .. } => self.button_mut(InputAction::Quit).is_down = true,
                Event::KeyUp {
                    keycode: Some(key), ..
                } => self.handle_sdl_key(*key, false),

                Event::KeyDown {
                    keycode: Some(key), ..
                } => self.handle_sdl_key(*key, true),

                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Left,
                    ..
                } => self.mouse_down.is_down = true,
                Event::MouseButtonUp {
                    mouse_btn: MouseButton::Left,
                    ..
                } => self.mouse_down.is_down = false,

                Event::MouseWheel { y, .. } => self.scroll += y,
                Event::MouseMotion { x, y, .. } => {
                    self.mouse.x = *x as i32;
                    self.mouse.y = *y as i32;
                },
                Event::ControllerAxisMotion { axis, value, .. } => {
                    let value = *value as f64 / 32767.0;
                    match axis {
                        Axis::LeftX => {
                            self.dir_move.x = value;
                        },
                        Axis::LeftY => {
                            self.dir_move.y = -value;
                        },
                        Axis::RightX => {
                            self.dir_look.x = value;
                        },
                        Axis::RightY => {
                            self.dir_look.y = -value;
                        },
                        _ => (),
                    }
                },
                _ => (),
            }
        }

        if self.button(InputAction::MoveUp).is_down {
            self.dir_move.y = 1.;
        }
        if self.button(InputAction::MoveDown).is_down {
            self.dir_move.y = -1.;
        }
        if self.button(InputAction::MoveRight).is_down {
            self.dir_move.x = 1.;
        }
        if self.button(InputAction::MoveLeft).is_down {
            self.dir_move.x = -1.;
        }

        if self.button(InputAction::LookUp).is_down {
            self.dir_look.y = 1.;
        }
        if self.button(InputAction::LookDown).is_down {
            self.dir_look.y = -1.;
        }
        if self.button(InputAction::LookRight).is_down {
            self.dir_look.x = 1.;
        }
        if self.button(InputAction::LookLeft).is_down {
            self.dir_look.x = -1.;
        }

        if self.button(InputAction::MoveUp).went_up() {
            self.dir_move.y = 0.0;
        }
        if self.button(InputAction::MoveDown).went_up() {
            self.dir_move.y = 0.0;
        }
        if self.button(InputAction::MoveRight).went_up() {
            self.dir_move.x = 0.0;
        }
        if self.button(InputAction::MoveLeft).went_up() {
            self.dir_move.x = 0.0;
        }

        if self.button(InputAction::LookUp).went_up() {
            self.dir_look.y = 0.0;
        }
        if self.button(InputAction::LookDown).went_up() {
            self.dir_look.y = 0.0;
        }
        if self.button(InputAction::LookRight).went_up() {
            self.dir_look.x = 0.0;
        }
        if self.button(InputAction::LookLeft).went_up() {
            self.dir_look.x = 0.0;
        }

        // self.dir_look = limit(self.dir_look);
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
        let act = self.sdl_key_to_action(key);
        let button = self.button_mut(act);
        button.is_down = down;
        if button.went_down() {
            eprintln!("action: {:?} = {}", act, if down { "down" } else { "up" });
        }
    }
}
