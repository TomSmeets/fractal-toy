use sdl2::{controller::Axis, controller::Button, event::*, keyboard::Keycode, mouse::MouseButton};

use fractal_toy::input::Input;
use fractal_toy::input::InputAction;
use fractal_toy::input::InputEvent;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct SDLInput {
    pub input: Input,
}

impl SDLInput {
    fn trnalsate_sdl_key(&mut self, key: Keycode) -> Option<InputAction> {
        Some(match key {
            Keycode::Q => InputAction::Quit,

            Keycode::W => InputAction::MoveUp,
            Keycode::S => InputAction::MoveDown,
            Keycode::D => InputAction::MoveLeft,
            Keycode::A => InputAction::MoveRight,

            Keycode::I => InputAction::ZoomIn,
            Keycode::K => InputAction::ZoomOut,

            Keycode::N => InputAction::NextFractal,
            Keycode::L => InputAction::IterInc,
            Keycode::J => InputAction::IterDec,

            Keycode::Num1 => InputAction::Pause,
            Keycode::Num2 => InputAction::Debug,
            Keycode::Num5 => InputAction::Save,
            Keycode::Num6 => InputAction::Load,
            _ => return None,
        })
    }

    fn handle_sdl_key(&mut self, key: Keycode, down: bool) {
        if let Some(act) = self.trnalsate_sdl_key(key) {
            self.input.events.push(InputEvent::Action(act, down));
        }
    }

    fn controller_button(&mut self, button: Button, down: bool) {
        let act = match button {
            Button::RightShoulder => Some(InputAction::IterInc),
            Button::LeftShoulder => Some(InputAction::IterDec),
            Button::A => Some(InputAction::NextFractal),
            Button::DPadUp => Some(InputAction::Debug),
            Button::DPadDown => Some(InputAction::Pause),
            Button::DPadLeft => Some(InputAction::Save),
            Button::DPadRight => Some(InputAction::Load),
            _ => None,
        };

        if let Some(act) = act {
            self.input.events.push(InputEvent::Action(act, down));
        }
    }

    pub fn handle_sdl(&mut self, events: &[Event]) {
        for e in events {
            match e {
                Event::Quit { .. } => self.input.quit = true,
                // Why does sdl-rs split these events? original events contain key state.
                Event::KeyUp {
                    keycode: Some(key), ..
                } => self.handle_sdl_key(*key, false),

                Event::KeyDown {
                    keycode: Some(key), ..
                } => self.handle_sdl_key(*key, true),

                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Left,
                    ..
                } => {
                    self.input.mouse_click = true;
                    self.input.mouse_down = true;
                },
                Event::MouseButtonUp {
                    mouse_btn: MouseButton::Left,
                    ..
                } => self.input.mouse_down = false,

                Event::MouseWheel { y, .. } => self.input.scroll += y,
                Event::MouseMotion { x, y, .. } => {
                    self.input.mouse.x = *x as i32;
                    self.input.mouse.y = *y as i32;
                },
                Event::ControllerAxisMotion { axis, value, .. } => {
                    let mut value = *value as f64 / 32767.0;

                    if value.abs() < 0.2 {
                        value = 0.0;
                    }

                    match axis {
                        Axis::LeftX => self.input.dir_move.x = value,
                        Axis::LeftY => self.input.dir_move.y = -value,
                        Axis::RightY => self.input.zoom = -value as f32,
                        _ => (),
                    }
                },
                Event::ControllerButtonDown { button, .. } => self.controller_button(*button, true),
                Event::ControllerButtonUp { button, .. } => self.controller_button(*button, false),
                Event::ControllerDeviceAdded { which, .. } => unsafe {
                    sdl2::sys::SDL_GameControllerOpen(*which as i32);
                },
                _ => (),
            }
        }
    }
}
