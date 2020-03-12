use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::input::Input;
use crate::math::*;
use crate::sdl::Sdl;

mod rect;
mod window;
pub use self::rect::Rect;
pub use self::window::Window;

pub struct DragAction {
    id: String,
    offset: V2i,
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
        let mouse_went_down = !self.was_down && input.mouse_down;
        let mouse_went_up = self.was_down && !input.mouse_down;

        self.was_down = input.mouse_down;

        if self.drag.is_some() && !input.mouse_down {
            self.drag = None;
        }

        // this is the top most window under the cursor
        let mut hot: Option<&String> = None;
        let mut active_changed = false;

        let mut windows: Vec<(&String, &mut Window)> =
            self.windows.iter_mut().filter(|(_, w)| w.visible).collect();
        windows.sort_by_key(|(_, w)| w.z_index);
        for (id, window) in windows.iter_mut() {
            if let Some(d) = &self.drag {
                if &&d.id == id {
                    window.rect.pos = input.mouse + d.offset;
                }
            }

            if hot.is_none() {
                let hover = window.is_inside(input.mouse);

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
                            offset: window.rect.pos - input.mouse,
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
