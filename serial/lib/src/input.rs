use sdl2::event::*;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;

use crate::math::*;

pub struct Input {
    pub mouse: V2i,
    pub mouse_down: bool,

    pub scroll: i32,
    pub dir_move: V2,
    pub dir_look: V2,
}

impl Input {
    pub fn new() -> Self {
        Input {
            mouse: V2i::zero(),
            mouse_down: false,
            scroll: 0,
            dir_move: V2::zero(),
            dir_look: V2::zero(),
        }
    }

    pub fn begin(&mut self) {
        self.scroll = 0;
    }

    // todo handle all events at once
    pub fn handle_sdl(&mut self, e: &Event) {
        match e {
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

            Event::MouseWheel { y, .. } => {
                self.scroll += y;
            },
            Event::MouseMotion { x, y, .. } => {
                self.mouse.x = *x as i32;
                self.mouse.y = *y as i32;
            },
            _ => {},
        }
    }

    fn handle_sdl_key(&mut self, key: Keycode, down: bool) {
        println!("key {:?} {}", key, if down { "down" } else { "up" });

        let v = if down { 1.0 } else { 0.0 };

        match key {
            Keycode::W => self.dir_move.y = v,
            Keycode::S => self.dir_move.y = -v,
            Keycode::D => self.dir_move.x = v,
            Keycode::A => self.dir_move.x = -v,

            Keycode::I => self.dir_look.y = v,
            Keycode::K => self.dir_look.y = -v,
            Keycode::L => self.dir_look.x = v,
            Keycode::J => self.dir_look.x = -v,
            _ => {},
        }
    }
}
