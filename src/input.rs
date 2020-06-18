use crate::fractal::Fractal;
use crate::math::*;
use crate::time::DeltaTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
// TODO(now): fractal should be a library, input should not exists here?
pub struct Input {
    pub mouse: V2i,
    pub old_mouse: V2i,

    // mouse drag in pixels
    pub drag: V2i,
    pub mouse_down: bool,
    pub mouse_click: bool,

    // Kind of like zoom, but instant and not smooth
    // TODO: maybe remove
    pub scroll: i32,
    pub zoom: f32,
    pub dir_move: V2,

    pub iter_inc: bool,
    pub iter_dec: bool,
    pub cycle: bool,

    // TODO: these are not part of fractal, maybe move out?
    pub quit: bool,
    pub debug: bool,
    pub pause: bool,
    pub load: bool,
    pub save: bool,
}

impl Default for Input {
    fn default() -> Self {
        Input::new()
    }
}

impl Input {
    pub fn new() -> Self {
        Input {
            mouse: V2i::zero(),
            old_mouse: V2i::zero(),

            mouse_down: false,
            mouse_click: false,
            drag: V2i::zero(),

            scroll: 0,
            zoom: 0.0,
            dir_move: V2::zero(),

            quit: false,

            iter_inc: false,
            iter_dec: false,
            cycle: false,

            debug: false,
            pause: false,
            load: false,
            save: false,
        }
    }

    pub fn begin(&mut self) {
        if self.mouse_down {
            self.drag = self.mouse - self.old_mouse;
        } else {
            self.drag = V2i::zero();
        }

        self.old_mouse = self.mouse;
        self.scroll = 0;
        self.mouse_click = false;
        self.iter_inc = false;
        self.iter_dec = false;
        self.cycle = false;
        self.load = false;
        self.save = false;
    }

    pub fn execute<T>(&mut self, fractal: &mut Fractal<T>, dt: DeltaTime) {
        if self.scroll != 0 {
            fractal.pos.zoom_in_at(0.3 * self.scroll as f64, self.mouse);
        }

        fractal.pos.translate({
            let mut p = dt.0 as f64 * self.dir_move * 2.0 * fractal.pos.size_in_pixels().x;
            p.y *= -1.0;
            to_v2i(p)
        });
        fractal.pos.zoom_in(dt.0 as f64 * self.zoom as f64 * 3.5);
        fractal.pos.translate(-self.drag);

        // TODO: in the future we want some kind of ui, or cli interface
        if self.iter_inc {
            fractal.params.iterations += 40;
            fractal.tiles.clear();
        }

        if self.iter_dec {
            fractal.params.iterations -= 40;
            fractal.params.iterations = fractal.params.iterations.max(3);
            fractal.tiles.clear();
        }

        if self.cycle {
            fractal.params.kind = fractal.params.kind.next();
            fractal.tiles.clear();
        }
    }
}

// TODO: no, move this to sdl, or even the entire input struct
#[cfg(feature = "platform-sdl")]
use sdl2::{controller::Axis, controller::Button, event::*, keyboard::Keycode, mouse::MouseButton};

#[cfg(feature = "platform-sdl")]
impl Input {
    fn handle_sdl_key(&mut self, key: Keycode, down: bool) {
        {
            let down_f = if down { 1.0 } else { 0.0 };
            let down_d = down_f as f64;
            match key {
                Keycode::Q => self.quit = true,

                Keycode::W => self.dir_move.y = 1.0 * down_d,
                Keycode::S => self.dir_move.y = -1.0 * down_d,
                Keycode::D => self.dir_move.x = 1.0 * down_d,
                Keycode::A => self.dir_move.x = -1.0 * down_d,

                Keycode::I => self.zoom = 1.0 * down_f,
                Keycode::K => self.zoom = -1.0 * down_f,
                _ => (),
            }
        }

        if down {
            match key {
                Keycode::N => self.cycle = true,
                Keycode::L => self.iter_inc = true,
                Keycode::J => self.iter_dec = true,

                Keycode::Num1 => self.pause = !self.pause,
                Keycode::Num2 => self.debug = !self.debug,
                Keycode::Num5 => self.save = true,
                Keycode::Num6 => self.load = true,
                _ => (),
            }
        }
    }

    fn controller_button(&mut self, button: Button, down: bool) {
        if down {
            match button {
                Button::RightShoulder => self.iter_inc = true,
                Button::LeftShoulder => self.iter_dec = true,
                Button::A => self.cycle = true,
                Button::DPadUp => self.debug = !self.debug,
                Button::DPadDown => self.pause = !self.pause,
                Button::DPadLeft => self.save = true,
                Button::DPadRight => self.load = true,
                _ => (),
            }
        }
    }

    pub fn handle_sdl(&mut self, events: &[Event]) {
        for e in events {
            match e {
                Event::Quit { .. } => self.quit = true,
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
                    self.mouse_click = true;
                    self.mouse_down = true;
                },
                Event::MouseButtonUp {
                    mouse_btn: MouseButton::Left,
                    ..
                } => self.mouse_down = false,

                Event::MouseWheel { y, .. } => self.scroll += y,
                Event::MouseMotion { x, y, .. } => {
                    self.mouse.x = *x as i32;
                    self.mouse.y = *y as i32;
                },
                Event::ControllerAxisMotion { axis, value, .. } => {
                    let mut value = *value as f64 / 32767.0;

                    if value.abs() < 0.2 {
                        value = 0.0;
                    }

                    match axis {
                        Axis::LeftX => self.dir_move.x = value,
                        Axis::LeftY => self.dir_move.y = -value,
                        Axis::RightY => self.zoom = -value as f32,
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
