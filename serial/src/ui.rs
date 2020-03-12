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
    pub z_index: i32,
    pub visible: bool,
    pub pos: (i32, i32),
    pub size: (i32, i32),
    pub color: [u8; 3],
}

impl Window {
    pub fn new() -> Self {
        Window {
            size: (80, 80),
            color: [255, 255, 255],
            ..Self::default()
        }
    }

    pub fn draw(&self, sdl: &mut Sdl) {
        let (x, y) = self.pos;
        let (w, h) = self.size;

        let mut r = SdlRect::new(x, y, w as u32, h as u32);
        sdl.canvas.set_draw_color(Color::RGB(0, 0, 0));
        sdl.canvas.fill_rect(r).unwrap();

        r.x += 4;
        r.y += 4;
        r.w -= 8;
        r.h -= 8;

        sdl.canvas
            .set_draw_color(Color::RGB(self.color[0], self.color[1], self.color[2]));
        sdl.canvas.fill_rect(r).unwrap();
    }

    pub fn is_inside(&self, x: i32, y: i32) -> bool {
        x >= self.pos.0
            && y >= self.pos.1
            && x < self.pos.0 + self.size.0
            && y < self.pos.1 + self.size.1
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
    // This is the selected window
    active: Option<String>,
    windows: HashMap<String, Window>,

    was_down: bool,
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
        windows.sort_by_key(|(_, w)| w.z_index);

        // this is the top most window under the cursor
        let mut hot: Option<&String> = None;

        let mouse_went_down = !self.was_down && input.mouse_down;
        let mouse_went_up = self.was_down && !input.mouse_down;
        self.was_down = input.mouse_down;

        let mut active_changed = false;
        for (id, window) in windows.iter_mut() {
            if !window.visible {
                continue;
            }

            let (x, y) = window.pos;

            if let Some(d) = &self.drag {
                if &&d.id == id {
                    window.pos.0 = input.mouse.x + d.offset.0;
                    window.pos.1 = input.mouse.y + d.offset.1;
                }
            }

            if hot.is_none() {
                let hover = window.is_inside(input.mouse.x, input.mouse.y);

                if hover {
                    hot = Some(id);
                    if mouse_went_down {
                        active_changed = true;
                        self.active = Some(id.clone());

                        // move window to top
                        window.z_index = -1;
                        // strat dragging
                        println!("Drag {}", id);
                        self.drag = Some(DragAction {
                            id: id.to_string(),
                            offset: (x - input.mouse.x, y - input.mouse.y),
                        });
                    }
                }
            }

            window.visible = false;
        }

        // TODO: make graphics implementation independet
        for (_, window) in windows.iter().rev() {
            window.draw(sdl);
        }

        // If we moved some window to front, move all other windows back
        if active_changed {
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
