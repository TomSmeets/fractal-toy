use crate::{
    math::{Rect, V2i},
    module::{input::Button, Input, Sdl},
};
use serde::{Deserialize, Serialize};

mod drawcmd;
mod storage;

use self::{drawcmd::DrawCmd, storage::Storage};
use sdl2::pixels::Color;

#[derive(Serialize, Deserialize)]
pub struct UIInput {
    mouse_pos: V2i,
    mouse_left: Button,
}

impl UIInput {
    pub fn new() -> Self {
        UIInput {
            mouse_pos: V2i::new(0, 0),
            mouse_left: Button::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct UI {
    storage: Storage,
    input: UIInput,
    hot: Option<Vec<String>>,

    #[serde(skip)]
    draw_list: Vec<DrawCmd>,
}

impl UI {
    pub fn update(&mut self, sdl: &mut Sdl, input: &Input, size: V2i) {
        for e in self.draw_list.drain(..) {
            match e {
                DrawCmd::Rect(rect, col) => {
                    sdl.canvas.set_draw_color(Color::RGB(
                        (col[0] * 255.0) as u8,
                        (col[1] * 255.0) as u8,
                        (col[2] * 255.0) as u8,
                    ));
                    sdl.canvas.fill_rect(rect.into_sdl()).unwrap();
                },
                _ => (),
            }
        }

        self.hot = None;
        self.input.mouse_pos = input.mouse;
        self.input.mouse_left = input.mouse_down;
    }

    pub fn new() -> Self {
        UI {
            input: UIInput::new(),
            storage: Storage::new(),
            draw_list: Vec::new(),
            hot: None,
        }
    }

    pub fn window<F: FnOnce(&mut UI) -> ()>(&mut self, name: &str, f: F) {
        self.storage.push_id(name);
        let hot = self.region(Rect {
            pos: V2i::new(0, 0),
            size: V2i::new(100, 100),
        });
        if hot {
            self.draw_list.push(DrawCmd::Rect(
                Rect {
                    pos: V2i::new(0, 0),
                    size: V2i::new(100, 100),
                },
                [1., 0., 0., 1.],
            ));
        }
        f(self);
        self.storage.pop_id();
    }

    pub fn button(&mut self, name: &str) -> bool {
        self.storage.push_id(name);
        let hot = self.region(Rect {
            pos: V2i::new(0, 0),
            size: V2i::new(40, 20),
        });
        if hot {
            println!("HOT");
        }
        let active = hot && self.input.mouse_left.is_down;
        self.storage.pop_id();
        active
    }

    pub fn region(&mut self, r: Rect) -> bool {
        self.draw_list.push(DrawCmd::Rect(r, [0.1, 0.1, 0.1, 1.]));
        if self.hot.is_none() {
            let inside = r.is_inside(self.input.mouse_pos);
            if inside {
                self.hot = Some(self.storage.id_stack.clone());
            }
            inside
        } else {
            false
        }
    }
}
