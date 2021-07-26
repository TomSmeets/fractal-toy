use std::{collections::BTreeMap, time::Instant};

use instant::Duration;

struct TimeEntry {
    dt_avrg: f32,
    dt_max: f32,
}

#[derive(Debug, Clone, Copy)]
struct DebugEvent {
    start_time: Instant,
    fame_duration:  Duration,
}

enum Item<T> {
    Push(&'static str, T),
    Pop,
}

pub struct TreeStorage<T> {
    items: Vec<Item<T>>,
}

impl<T> TreeStorage<T> {
    pub fn new() -> Self { TreeStorage { items: Vec::new() }}

    pub fn restart(&mut self) {
        // TODO: keep items
        self.items.clear();
    }

    pub fn push(&mut self, name: &'static str, def: T) -> &mut T {
        self.items.push(Item::Push(name, def));
        match self.items.last_mut().unwrap() {
            Item::Push(_, t) => t,
            Item::Pop => unreachable!(),
        }
    }

    pub fn pop(&mut self) -> &mut T {
        let pop_ix = self.items.len();
        self.items.push(Item::Pop);
        let push_ix = Self::find_matching_push(&self.items, pop_ix);
        match &mut self.items[push_ix] {
            Item::Push(_, t) => t,
            Item::Pop => unreachable!(),
        }
    }

    fn find_matching_push(items: &[Item<T>], pop_ix: usize) -> usize {
        let mut depth = 0;
        let mut ix = pop_ix;

        loop {
            match items[ix] {
                Item::Push(_, _) => depth -= 1,
                Item::Pop        => depth += 1,
            }

            if depth == 0 {
                return ix;
            }

            if ix == 0 {
                panic!("OH NO");
            }

            ix = ix - 1;
        }
    }
}

pub struct Debug {
    info: String,
    out: String,
    
    last_frame_events: TreeStorage<DebugEvent>,
    events: TreeStorage<DebugEvent>,
}

impl Debug {
    pub fn new() -> Self {
        Debug {
            info: String::new(),
            out: String::new(),

            last_frame_events: TreeStorage::new(),
            events: TreeStorage::new(),
        }
    }

    pub fn begin(&mut self) {
        std::mem::swap(&mut self.events, &mut self.last_frame_events);
        self.events.restart();

        std::mem::swap(&mut self.out, &mut self.info);
        self.info.clear();
    }

    pub fn push(&mut self, name: &'static str) {
        let time = Instant::now();
        let ev = self.events.push(name, DebugEvent {
            start_time: time,
            fame_duration: Duration::ZERO,
        });
        ev.start_time = time;
    }

    pub fn pop(&mut self) {
        let ev = self.events.pop();
        let time = Instant::now();
        ev.fame_duration = time - ev.start_time;
    }

    pub fn draw(&mut self) -> String {
        let mut result = std::mem::take(&mut self.out);
        result.push_str("-----------\n");

        let mut depth = 0;
        for e in self.last_frame_events.items.iter() {
            match e {
                Item::Push(name, ev) => {
                    result.push_str(&format!(
                        "{:6.2} µs | ",
                        ev.fame_duration.as_micros()
                    ));
                    for _ in 0..depth {
                        result.push_str("  ");
                    }
                    result.push_str(name);
                    result.push_str("\n");

                    depth += 1;
                }

                Item::Pop => {
                    depth -= 1;
                }
            }
        }

        result
    }

    pub fn print(&mut self, s: &str) {
        self.info.push_str(s);
        self.info.push_str("\n");
    }
}
