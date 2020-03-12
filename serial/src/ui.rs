use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::input::Input;
use crate::math::*;
use crate::sdl::Sdl;

mod collection;
mod rect;
mod window;

pub use self::collection::Collection;
pub use self::rect::Rect;
pub use self::window::Window;

enum DragActionType {
    Drag,
    Resize,
}

pub struct DragAction {
    id: String,
    offset: V2i,
    mode: DragActionType,
}

#[derive(Serialize, Deserialize, Default)]
pub struct UI {
    #[serde(skip)]
    drag: Option<DragAction>,
    // This is the selected window
    active: Option<String>,
    windows: Collection<Window>,

    was_down: bool,
}

impl UI {
    pub fn new() -> Self {
        UI::default()
    }

    pub fn update(&mut self, sdl: &mut Sdl, input: &Input) {
        if self.drag.is_some() && input.mouse_down.went_up() {
            self.drag = None;
        }

        // this is the top most window under the cursor
        let mut hot: Option<&str> = None;
        let mut active_changed = false;

        let mut windows: Vec<_> = self
            .windows
            .content
            .iter_mut()
            .map(|i| (&i.id, &mut i.value))
            .filter(|(_, w)| w.visible)
            .collect();
        windows.sort_by_key(|(_, w)| w.z_index);
        for (id, window) in windows.iter_mut() {
            let id: &str = id;
            let window: &mut Window = window;

            if let Some(d) = &self.drag {
                if d.id == id {
                    match d.mode {
                        DragActionType::Drag => window.rect.pos = input.mouse + d.offset,
                        DragActionType::Resize => {
                            window.rect.size = input.mouse - window.rect.pos;
                            if window.rect.size.x < 0 {
                                window.rect.size.x = 0;
                            }
                            if window.rect.size.y < 20 {
                                window.rect.size.y = 20;
                            }
                        },
                    }
                }
            }

            if hot.is_none() && window.is_inside(input.mouse) {
                // this window is the first under the cursor
                hot = Some(id);
                if input.mouse_down.went_down() {
                    // we clicked somewhere inside the window
                    active_changed = true;
                    self.active = Some(id.to_string());

                    // move window to top
                    window.z_index = -1;

                    // strat dragging
                    println!("Drag {}", id);

                    if window.header_rect().is_inside(input.mouse) {
                        self.drag = Some(DragAction {
                            id: id.to_string(),
                            offset: window.rect.pos - input.mouse,
                            mode: DragActionType::Drag,
                        });
                    }

                    if window.resize_handle_rect().is_inside(input.mouse) {
                        self.drag = Some(DragAction {
                            id: id.to_string(),
                            offset: window.rect.pos - input.mouse,
                            mode: DragActionType::Resize,
                        });
                    }
                }
            }

            window.visible = false;
        }

        // If we moved some window to front, move all other windows back
        if active_changed {
            for (i, w) in windows.iter_mut() {
                w.z_index += 1;
                println!("{}: z = {}", i, w.z_index);
            }
        }

        // TODO: make graphics implementation independet
        // all windows are drawn from bottom to top, so we have to iterate in reverse
        for (_, window) in windows.iter().rev() {
            window.draw(sdl);
        }
    }

    pub fn window(&mut self, title: &str) -> &mut Window {
        // NOTE: probably want to create a gneeric data structure for this operation
        // NOTE: the hashmap could be improved by the fact that the windows will be called in the
        // NOTE: same order almost every time so a plain vector with linear search starting
        // NOTE: from the current position could be just as effective
        let w = self.windows.item(title, Window::new);
        // move into Collection
        w.visible = true;
        w
    }
}
