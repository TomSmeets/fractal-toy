use crate::Config;
use crate::Input;
use crate::InputAction;
use crate::InputEvent;
use crate::Vector2;
use crate::Viewport;
use cgmath::InnerSpace;
use sdl2::controller::Axis;
use sdl2::controller::Button;
use sdl2::event::*;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct SDLInput {
    pub mouse_down: bool,
    pub mouse: Vector2<i32>,

    pub controller_left: [f32; 2],
    pub controller_right: [f32; 2],

    pub wsad_state: [bool; 4],
    pub ikjl_state: [bool; 4],
}

impl SDLInput {
    pub fn new() -> Self {
        SDLInput {
            wsad_state: [false; 4],
            ikjl_state: [false; 4],

            controller_left: [0.0; 2],
            controller_right: [0.0; 2],
            mouse: Vector2::new(0, 0),
            mouse_down: false,
        }
    }
    fn handle_sdl_key(&mut self, input: &mut Input, key: Keycode, down: bool) {
        match key {
            Keycode::W => self.wsad_state[0] = down,
            Keycode::S => self.wsad_state[1] = down,
            Keycode::A => self.wsad_state[2] = down,
            Keycode::D => self.wsad_state[3] = down,

            Keycode::I => self.ikjl_state[0] = down,
            Keycode::K => self.ikjl_state[1] = down,

            Keycode::J if down => input.iter += 1,
            Keycode::L if down => input.iter -= 1,
            Keycode::N if down => input.next += 1,

            Keycode::Num1 if down => input.pause = true,
            Keycode::Num2 if down => input.debug = true,
            Keycode::Num5 if down => input.save = true,
            Keycode::Num6 if down => input.load = true,
            _ => (),
        }
    }

    #[rustfmt::skip]
    fn controller_button(&mut self, input: &mut Input, button: Button, down: bool) {
        match button {
            Button::RightShoulder if down => input.iter += 1,
            Button::LeftShoulder  if down => input.iter -= 1,
            Button::A             if down => input.next += 1,
            Button::DPadUp        if down => input.debug = true,
            Button::DPadDown      if down => input.pause = true,
            Button::DPadLeft      if down => input.save = true,
            Button::DPadRight     if down => input.load = true,
            _ => (),
        };
    }

    // I like the allignment here
    #[rustfmt::skip]
    pub fn handle_sdl(&mut self, events: &[Event]) -> Input {
        let mut input = Input::new();

        for e in events {
            match e {
                Event::Quit { .. } => input.quit = true,

                // Why does sdl-rs split these events? original events contain key state.
                Event::KeyUp   { keycode: Some(key), .. } => self.handle_sdl_key(&mut input, *key, false),
                Event::KeyDown { keycode: Some(key), .. } => self.handle_sdl_key(&mut input, *key, true),

                Event::MouseButtonDown { mouse_btn: MouseButton::Left, .. } => {
                    input.mouse_click = true;
                    self.mouse_down = true;
                },

                Event::MouseButtonUp { mouse_btn: MouseButton::Left, .. } => self.mouse_down = false,

                Event::MouseWheel { y, .. } => input.scroll += y,
                Event::MouseMotion { x, y, xrel, yrel, .. } => {
                    self.mouse.x = *x as i32;
                    self.mouse.y = *y as i32;

                    input.drag.x += xrel;
                    input.drag.y += yrel;
                },
                Event::ControllerAxisMotion { axis, value, .. } => {
                    let mut value = *value as f32 / 32767.0;

                    // TODO: maybe move out?
                    if value.abs() < 0.2 {
                        value = 0.0;
                    }

                    match axis {
                        Axis::LeftX  => self.controller_left[0]  = value,
                        Axis::LeftY  => self.controller_left[1]  = value,
                        Axis::RightX => self.controller_right[0] = value,
                        Axis::RightY => self.controller_right[1] = value,
                        _ => (),
                    }
                },
                Event::ControllerButtonDown { button, .. } => self.controller_button(&mut input, *button, true),
                Event::ControllerButtonUp   { button, .. } => self.controller_button(&mut input, *button, false),

                Event::ControllerDeviceAdded { which, .. } => unsafe { sdl2::sys::SDL_GameControllerOpen(*which as i32); },

                Event::Window { win_event, .. } => match win_event {
                    sdl2::event::WindowEvent::Resized(x, y) => {
                        let window_size = Vector2::new((*x as u32).max(1), (*y as u32).max(1));
                        input.resize = Some(window_size);
                    },
                    _ => (),
                },
                _ => (),
            }
        }

        let mut move_dir = Vector2::new(0.0, 0.0);
        if self.wsad_state[0] { move_dir.y += 1.0; }
        if self.wsad_state[1] { move_dir.y -= 1.0; }
        if self.wsad_state[2] { move_dir.x -= 1.0; }
        if self.wsad_state[3] { move_dir.x += 1.0; }
        input.dir_move = move_dir.normalize();

        let mut look_dir = Vector2::new(0.0, 0.0);
        if self.ikjl_state[0] { look_dir.y += 1.0; }
        if self.ikjl_state[1] { look_dir.y -= 1.0; }
        if self.ikjl_state[2] { look_dir.x -= 1.0; }
        if self.ikjl_state[3] { look_dir.x += 1.0; }
        input.zoom = look_dir.y;

        input.mouse = self.mouse;
        input.mouse_down = self.mouse_down;

        input
    }
}
