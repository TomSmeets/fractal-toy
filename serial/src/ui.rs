use sdl2::pixels::*;

use std::collections::hash_map::{Entry, HashMap};

use serde::{Serialize, Deserialize};

use crate::input::Input;
use crate::sdl::Sdl;

use std::cmp::Eq;
use std::hash::Hash;

type SdlRect = sdl2::rect::Rect;

pub struct Rect {
    pos: (i32, i32),
    size: (i32, i32),
    color: Color,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
#[derive(Serialize, Deserialize)]
pub struct ElementState {
    pos:  (i32, i32),
    size: (i32, i32),
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
#[derive(Serialize, Deserialize, Default)]
pub struct ElementID(Vec<String>);

#[derive(Serialize, Deserialize, Default)]
pub struct UIInput {
    mouse: (i32, i32),
    down: bool,
    click: bool,
    click_begin: bool,
}

#[derive(
    Serialize, Deserialize, Default
)]
pub struct UIStateMap (HashMap<ElementID, ElementState>);

impl UIStateMap {
    pub fn get_state_or(&mut self, id: &ElementID, s: ElementState) -> &mut ElementState {
        self.0.entry(id.clone()).or_insert(s)
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct UI {
    input: UIInput,

    hot_item: Option<ElementID>,

    current_item: ElementID,

    state: UIStateMap,

    #[serde(skip)]
    rects: Vec<Rect>,

    tmp_y: i32,
    drag_offset: (i32, i32),
}

impl UI {
    pub fn new() -> Self {
        UI::default()
    }

    pub fn update(&mut self, sdl: &mut Sdl, input: &Input) {
        println!("Active item: {:?}", self.hot_item);
        self.input.mouse.0 = input.mouse.x;
        self.input.mouse.1 = input.mouse.y;
        self.input.click = self.input.down && !input.mouse_down;
        self.input.click_begin = !self.input.down && input.mouse_down;
        self.input.down = input.mouse_down;
        self.hot_item = None;

        for Rect { pos, size, color } in self.rects.iter().rev() {
            sdl.canvas.set_draw_color(*color);
            sdl.canvas
                .fill_rect(SdlRect::new(pos.0, pos.1, size.0 as u32, size.1 as u32))
                .unwrap();
        }
        self.rects.clear();
        self.tmp_y = 0;
    }

    pub fn push_id(&mut self, id: &str) {
        self.current_item.0.push(id.to_string());
    }

    pub fn pop_id(&mut self) {
        self.current_item.0.pop();
    }

    // TODO: use FnOnce ?
    pub fn window_begin(&mut self, name: &str) {
        self.push_id(name);

        let s = {
            let new_y = self.tmp_y;
            self.tmp_y += 40;
            ElementState { pos: (10, new_y), size: (80, 80) }
        };

        let state = self.state.get_state_or(&self.current_item, s.clone());
        let (x, y) = state.pos;
        let (w, h) = state.size;

        let is_inside = self.region(x, y, w, h);
        let is_hot = if is_inside && self.hot_item.is_none() {
            self.hot_item = Some(self.current_item.clone());
            true
        } else {
            false
        };

        let color = if is_hot {
            Color::RGB(255, 0, 0)
        } else {
            Color::RGB(0, 0, 255)
        };

        if is_hot && self.input.click_begin {
            self.drag_offset.0 = x - self.input.mouse.0;
            self.drag_offset.1 = y - self.input.mouse.1;
        }

        if is_hot && self.input.down {
            println!("clicked {}", name);
            let s = self.state.get_state_or(&self.current_item, s.clone());
            s.pos.0 = self.input.mouse.0 + self.drag_offset.0;
            s.pos.1 = self.input.mouse.1 + self.drag_offset.1;
        }

        self.rects.push(Rect {
            pos: (x + 2, y + 2),
            size: (w - 4, h - 4),
            color,
        });

        self.rects.push(Rect {
            pos: (x, y),
            size: (w, h),
            color: Color::RGB(0, 0, 0),
        });

    }

    pub fn window_end(&mut self) {
        self.pop_id();
    }

    pub fn region(&mut self, x: i32, y: i32, w: i32, h: i32) -> bool {
        self.input.mouse.0 >= x && self.input.mouse.1 >= y && self.input.mouse.0 < x + w && self.input.mouse.1 < y + h
    }
}
