use crate::input::Button;
use crate::input::Input;
use crate::math::*;
use crate::sdl::Sdl;
use sdl2::pixels::Color;

mod collection;
mod rect;
mod window;

pub use self::collection::Collection;
pub use self::rect::Rect;

enum DragActionType {
    Drag,
    Resize,
}

pub struct DragAction {
    id: String,
    offset: V2i,
    mode: DragActionType,
}

pub struct UIRect {
    rect: Rect,
    color: [u8; 3],
    text: String,
}

#[derive(Clone)]
pub struct UIState {
    pub rect: Rect,
}

impl UIState {
    pub fn new() -> UIState {
        UIState {
            rect: Rect {
                pos: V2i::new(0, 0),
                size: V2i::new(800, 800),
            },
        }
    }
}

pub struct UI {
    pub mouse_pos: V2i,
    pub mouse_down: Button,

    pub rects: Vec<UIRect>,

    pub state: UIState,
    pub stack: Vec<UIState>,

    pub current: Vec<String>,

    // TODO
    pub data: Collection<V2i>,
    pub drag: Option<DragAction>,
}

fn draw_rect(sdl: &mut Sdl, r: Rect, color: [u8; 3]) {
    let r = r.into_sdl();
    let mut r2 = r;
    r2.x -= 2;
    r2.y -= 2;
    r2.w += 4;
    r2.h += 4;

    sdl.canvas.set_draw_color(Color::RGB(0, 0, 0));
    sdl.canvas.fill_rect(r2).unwrap();

    sdl.canvas
        .set_draw_color(Color::RGB(color[0], color[1], color[2]));
    sdl.canvas.fill_rect(r).unwrap();
}

impl UI {
    pub fn new() -> UI {
        UI {
            mouse_pos: V2i::new(0, 0),
            mouse_down: Button::new(),
            rects: Vec::new(),
            state: UIState::new(),
            stack: Vec::new(),
            data: Collection::new(),
            current: Vec::new(),
            drag: None,
        }
    }

    pub fn update(&mut self, sdl: &mut Sdl, input: &Input) {
        self.mouse_pos = input.mouse;
        self.mouse_down = input.mouse_down;
        self.state.rect.pos = V2i::new(0, 0);

        if self.drag.is_some() && !self.mouse_down.is_down {
            self.drag = None;
        }

        for r in self.rects.iter_mut() {
            draw_rect(sdl, r.rect, r.color);

            {
                let (mut rect, texture) = sdl.make_text(&r.text, 20.0);
                rect.x = r.rect.pos.x + r.rect.size.x / 2 - rect.w / 2;
                rect.y = r.rect.pos.y + r.rect.size.y / 2 - rect.h / 2;
                sdl.draw_rgba(rect, &texture);
            }
        }
        self.rects.clear();
    }

    pub fn region(&mut self, title: &str) -> bool {
        let size = V2i::new(self.state.rect.size.x, 40);
        let pos = self.state.rect.pos;

        self.state.rect.pos.y += size.y + 5;

        let r = Rect { pos, size };

        let hot = r.is_inside(self.mouse_pos);

        let color = if hot { [128, 0, 0] } else { [128, 128, 128] };

        self.rects.push(UIRect {
            rect: r,
            text: title.to_string(),
            color,
        });

        hot
    }

    pub fn button(&mut self, title: &str) -> bool {
        self.region(title) && self.mouse_down.went_down()
    }

    pub fn vsplit<F: FnOnce(&mut UI), G: FnOnce(&mut UI)>(&mut self, f: F, g: G) {
        let y_old = self.state.rect.pos.y;
        let s_old = self.state.rect.size.x;
        let x_old = self.state.rect.pos.x;
        let mut y_new = y_old;
        self.state.rect.size.x /= 2;

        f(self);
        y_new = y_new.max(self.state.rect.pos.y);
        self.state.rect.pos.y = y_old;
        self.state.rect.pos.x += self.state.rect.size.x;
        g(self);
        y_new = y_new.max(self.state.rect.pos.y);

        self.state.rect.size.x = s_old;
        self.state.rect.pos.x = x_old;
        self.state.rect.pos.y = y_new;
    }

    pub fn window_start<F: FnOnce(&mut UI)>(&mut self, title: &str, f: F) {
        let o = self.state.rect.pos;
        self.state.rect.pos += V2i::new(20, 20);

        self.current.push(title.to_string());

        if let Some(act) = &self.drag {
            if act.id == title {
                let pos = self.data.item(title, || o);
                *pos = self.mouse_pos + act.offset;
            }
        }

        let pos = *self.data.item(title, || o);
        let size = V2i::new(200, 200);
        let rect = Rect { pos, size };

        let hot = rect.is_inside(self.mouse_pos);

        self.rects.push(UIRect {
            rect,
            text: title.to_string(),
            color: [0, 0, 128],
        });

        let state = self.state.clone();
        self.state.rect.size = size - V2i::new(8, 8);
        self.state.rect.pos = pos + V2i::new(4, 4);

        let header = self.region(title);

        if header && self.mouse_down.is_down {
            if self.drag.is_none() {
                self.drag = Some(DragAction {
                    id: title.to_string(),
                    offset: pos - self.mouse_pos,
                    mode: DragActionType::Drag,
                })
            }
        }

        f(self);

        self.state = state;
    }

    pub fn window_stop(&mut self) {}
}
