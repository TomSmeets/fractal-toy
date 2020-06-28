use crate::fractal::Fractal;
use crate::math::*;
use crate::time::DeltaTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
// TODO: fractal should be a library, input should not exists here?
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

    // TODO: these are not part of fractal, maybe move out?
    pub quit: bool,
    pub debug: bool,
    pub pause: bool,
    pub load: bool,
    pub save: bool,

    pub events: Vec<InputEvent>,
}

// These can be seriealized
// We should handle all of these
#[derive(Serialize, Deserialize, Clone)]
pub enum MouseEvent {
    Move(V2i),
    Button(u32, bool),
    Wheel(i32),
}

// TODO: In the future there should be a key binding for these
#[derive(Serialize, Deserialize, Clone)]
pub enum InputAction {
    Quit,
    Debug,
    Pause,
    Load,
    Save,
    IterInc,
    IterDec,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    ZoomIn,
    ZoomOut,
    NextFractal,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum InputEvent {
    Mouse(MouseEvent),

    // think keyboard key, but named
    Action(InputAction, bool),
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

            debug: false,
            pause: false,
            load: false,
            save: false,
            events: Vec::new(),
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
        self.load = false;
        self.save = false;
    }

    // why not?
    // How do we know how if a key is being held down?
    pub fn run_events<T>(&mut self, fractal: &mut Fractal<T>) {
        for ev in self.events.iter() {
            match ev {
                InputEvent::Action(act, down) => {
                    if *down {
                        match act {
                            InputAction::Quit => self.quit = true,
                            InputAction::Debug => self.debug = !self.debug,
                            InputAction::Pause => self.pause = !self.pause,
                            InputAction::Load => self.load = true,
                            InputAction::Save => self.save = true,
                            InputAction::NextFractal => {
                                fractal.params.kind = fractal.params.kind.next();
                                fractal.reload();
                            },
                            InputAction::IterInc => {
                                fractal.params.iterations += 40;
                                fractal.reload()
                            },
                            InputAction::IterDec => {
                                fractal.params.iterations -= 40;
                                fractal.params.iterations = fractal.params.iterations.max(3);
                                fractal.reload()
                            },
                            _ => (),
                        };
                    }
                    {
                        let down_d = if *down { 1.0 } else { 0.0 };
                        match act {
                            InputAction::MoveUp => self.dir_move.y = down_d,
                            InputAction::MoveDown => self.dir_move.y = -down_d,
                            InputAction::MoveLeft => self.dir_move.x = down_d,
                            InputAction::MoveRight => self.dir_move.x = -down_d,
                            InputAction::ZoomIn => self.zoom = 1.0 * down_d as f32,
                            InputAction::ZoomOut => self.zoom = -1.0 * down_d as f32,
                            _ => (),
                        };
                    }
                },
                _ => (),
            }
        }

        self.events.clear();
    }

    // TODO: in the future we want some kind of ui, or cli interface
    pub fn execute<T>(&mut self, fractal: &mut Fractal<T>, dt: DeltaTime) {
        self.run_events(fractal);

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
    }
}

// TODO: no, move this to sdl, or even the entire input struct
#[cfg(feature = "platform-sdl")]
use sdl2::{controller::Axis, controller::Button, event::*, keyboard::Keycode, mouse::MouseButton};

#[cfg(feature = "platform-sdl")]
impl Input {
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
            self.events.push(InputEvent::Action(act, down));
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
            self.events.push(InputEvent::Action(act, down));
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
