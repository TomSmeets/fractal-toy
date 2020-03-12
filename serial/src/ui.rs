use sdl2::pixels::*;

use std::collections::hash_map::{Entry, HashMap};

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

pub struct ElementState {
    pos: (i32, i32),
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct ElementID(Vec<String>);

pub struct UI {
    mouse: (i32, i32),

    down: bool,
    click:   bool,

    hot_item: Option<ElementID>,

    current_item: ElementID,

    state: HashMap<ElementID, ElementState>,
    rects: Vec<Rect>,


    tmp_y: i32,
}

impl UI {
    pub fn new() -> Self {
        UI {
            rects: Vec::new(),
            state: HashMap::new(),
            down: false,
            click: false,
            mouse: (0, 0),
            hot_item: None,
            current_item: ElementID(Vec::new()),
            tmp_y: 0,
        }
    }

    pub fn update(&mut self, sdl: &mut Sdl, input: &Input) {
        println!("Active item: {:?}", self.hot_item);
        self.mouse.0 = input.mouse.x;
        self.mouse.1 = input.mouse.y;
        self.click = self.down && !input.mouse_down;
        self.down = input.mouse_down;
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

    pub fn get_state(&mut self) -> Entry<'_, ElementID, ElementState> {
        self.state.entry(self.current_item.clone())
    }

    // TODO: use FnOnce ?
    pub fn window_begin(&mut self, name: &str) {
        self.push_id(name);

        let w = 80;
        let h = 80;

        let new_y = self.tmp_y;
        self.tmp_y += 40;

        let state = self
            .get_state()
            .or_insert_with(|| ElementState { pos: (10, new_y) });
        let (x, y) = state.pos;

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


        if is_hot && self.click {
            println!("clicked {}", name);
            self
                .get_state().and_modify(|s| s.pos.0 += 10)
                ;
        }

        self.rects.push(Rect {
            pos: (x, y),
            size: (w, h),
            color,
        });
    }

    pub fn window_end(&mut self) {
        self.pop_id();
    }

    pub fn region(&mut self, x: i32, y: i32, w: i32, h: i32) -> bool {
        self.mouse.0 >= x && self.mouse.1 >= y && self.mouse.0 < x + w && self.mouse.1 < y + h
    }
}
