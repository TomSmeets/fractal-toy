use sdl2::pixels::*;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::input::Input;
use crate::sdl::Sdl;

use std::cmp::Eq;
use std::hash::Hash;

type SdlRect = sdl2::rect::Rect;

#[derive(Serialize, Deserialize, Default)]
pub struct Window {
    z_index: i32,
    visible: bool,
    pos: (i32, i32),
    size: (i32, i32),
    active: bool,
}

impl Window {
    pub fn new() -> Self {
        Window {
            size: (80, 80),
            ..Self::default()
        }
    }
}

pub struct DragAction {
    id: String,
    offset: (i32, i32),
}

#[derive(Serialize, Deserialize, Default)]
pub struct UI {
    #[serde(skip)]
    drag: Option<DragAction>,
    windows: HashMap<String, Window>,
}

impl UI {
    pub fn new() -> Self {
        UI::default()
    }

    pub fn update(&mut self, sdl: &mut Sdl, input: &Input) {
        if self.drag.is_some() && !input.mouse_down {
            self.drag = None;
        }

        let mut windows: Vec<(&String, &mut Window)> = self.windows.iter_mut().collect();
        let mut focus = false;
        windows.sort_by_key(|(_, w)| w.z_index);
        for (id, window) in windows.iter_mut() {
            if !window.visible {
                continue;
            }

            let (x, y) = window.pos;
            let (w, h) = window.size;

            if let Some(d) = &self.drag {
                if &&d.id == id {
                    window.pos.0 = input.mouse.x + d.offset.0;
                    window.pos.1 = input.mouse.y + d.offset.1;
                }
            }

            {
                let hover = input.mouse.x >= x
                    && input.mouse.y >= y
                    && input.mouse.x < x + w
                    && input.mouse.y < y + h;

                if self.drag.is_none() && hover && input.mouse_down {
                    window.z_index = 0;
                    focus = true;
                    println!("Drag {}", id);
                    self.drag = Some(DragAction {
                        id: id.to_string(),
                        offset: (x - input.mouse.x, y - input.mouse.y),
                    });
                }
            }

            window.visible = false;
        }

        for (_, window) in windows.iter().rev() {
            let (x, y) = window.pos;
            let (w, h) = window.size;

            let mut r = SdlRect::new(x, y, w as u32, h as u32);
            sdl.canvas.set_draw_color(Color::RGB(0, 0, 0));
            sdl.canvas.fill_rect(r).unwrap();

            r.x += 4;
            r.y += 4;
            r.w -= 8;
            r.h -= 8;

            sdl.canvas.set_draw_color(Color::RGB(255, 255, 255));
            sdl.canvas.fill_rect(r).unwrap();
        }

        if focus {
            for (i, w) in windows.iter_mut() {
                w.z_index += 1;

                println!("{}: z = {}", i, w.z_index);
            }
        }
    }

    pub fn window(&mut self, title: &str) -> &mut Window {
        // probably want to create a gneeric data structure for this operation
        let w = self
            .windows
            .entry(title.to_string())
            .or_insert_with(|| Window::new());
        w.visible = true;
        w
    }
}
