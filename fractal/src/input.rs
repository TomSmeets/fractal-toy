use crate::math::*;
use crate::Config;
use crate::Vector2;
use crate::Viewport;
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

    pub iter: i32,
    pub next: i32,

    pub resize: Option<Vector2<u32>>,

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

            iter: 0,
            next: 0,
            resize: None,
        }
    }

    pub fn move_viewport(&self, dt: f32, vp: &mut Viewport) {
        if let Some(sz) = self.resize {
            vp.resize(sz);
        }

        if self.scroll != 0 {
            vp.zoom_in_at(0.3 * self.scroll as f64, self.mouse);
        }

        vp.translate({
            let mut p = dt as f64 * self.dir_move * 2.0 * vp.size_in_pixels().x;
            p.y *= -1.0;
            crate::V2i::new(p.x as i32, p.y as i32)
        });

        vp.zoom_in(dt as f64 * self.zoom as f64 * 3.5);

        if self.mouse_down {
            vp.translate(-self.drag);
        }
    }

    pub fn update_config(&self, _cfg: &mut Config) {}
}
