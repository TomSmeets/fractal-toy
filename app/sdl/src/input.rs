use crate::main2::Config;
use fractal_toy::Input;
use fractal_toy::InputAction;
use fractal_toy::InputEvent;
use fractal_toy::Vector2;
use fractal_toy::Viewport;
use sdl2::controller::Axis;
use sdl2::controller::Button;
use sdl2::event::*;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct SDLInput {
    pub dt: f32,
    pub input: Input,
    pub resize: Option<Vector2<u32>>,
}

impl SDLInput {
    pub fn new(dt: f32) -> Self {
        SDLInput {
            dt,
            input: Input::new(),
            resize: None,
        }
    }
    pub fn is_quit(&self) -> bool {
        self.input.quit
    }

    // TODO: Remove mut
    pub fn move_viewport(&mut self, vp: &mut Viewport) {
        if let Some(sz) = self.resize {
            vp.resize(sz);
        }

        for ev in self.input.events.iter() {
            match ev {
                InputEvent::Action(act, down) => {
                    let down_d = if *down { 1.0 } else { 0.0 };
                    match act {
                        InputAction::MoveUp => self.input.dir_move.y = down_d,
                        InputAction::MoveDown => self.input.dir_move.y = -down_d,
                        InputAction::MoveLeft => self.input.dir_move.x = down_d,
                        InputAction::MoveRight => self.input.dir_move.x = -down_d,
                        InputAction::ZoomIn => self.input.zoom = 1.0 * down_d as f32,
                        InputAction::ZoomOut => self.input.zoom = -1.0 * down_d as f32,
                        _ => (),
                    };
                },
                _ => (),
            }
        }

        if self.input.scroll != 0 {
            vp.zoom_in_at(0.3 * self.input.scroll as f64, self.input.mouse);
        }

        vp.translate({
            let mut p = self.dt as f64 * self.input.dir_move * 2.0 * vp.size_in_pixels().x;
            p.y *= -1.0;
            fractal_toy::V2i::new(p.x as i32, p.y as i32)
        });
        vp.zoom_in(self.dt as f64 * self.input.zoom as f64 * 3.5);
        vp.translate(-self.input.drag);

        self.input.events.clear();
    }

    pub fn update_config(&self, cfg: &mut Config) {}

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
        self.input.begin();

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
