use std::time::Instant;

use instant::Duration;

struct TimeEntry {
    dt_avrg: f32,
    dt_max: f32,
}

#[derive(Debug, Clone, Copy)]
struct DebugEvent {
    start_time: Instant,
    fame_duration: Duration,
    table_index: u32,
    table: [u32; 180],
}

enum Item<T> {
    Push(&'static str, T),
    Pop,
}

pub struct TreeStorage<T> {
    items: Vec<Item<T>>,
    prev_items: Vec<Item<T>>,
}

impl<T: Clone> TreeStorage<T> {
    pub fn new() -> Self {
        TreeStorage {
            items: Vec::new(),
            prev_items: Vec::new(),
        }
    }

    pub fn restart(&mut self) {
        std::mem::swap(&mut self.items, &mut self.prev_items);
        self.items.clear();
    }

    pub fn push(&mut self, name: &'static str, def: T) -> &mut T {
        let ix = self.items.len();
        match self.prev_items.get(ix) {
            Some(Item::Push(n, t)) if n == &name => self.items.push(Item::Push(name, t.clone())),
            _ => self.items.push(Item::Push(name, def)),
        }
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
                Item::Pop => depth += 1,
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

    events: TreeStorage<DebugEvent>,
}

impl Debug {
    pub fn new() -> Self {
        Debug {
            info: String::new(),
            out: String::new(),
            events: TreeStorage::new(),
        }
    }

    pub fn begin(&mut self) {
        self.events.restart();

        std::mem::swap(&mut self.out, &mut self.info);
        self.info.clear();
    }

    pub fn push(&mut self, name: &'static str) {
        let time = Instant::now();
        let ev = self.events.push(name, DebugEvent {
            start_time: time,
            fame_duration: Duration::ZERO,
            table_index: 0,
            table: [0; 180],
        });
        ev.start_time = time;
    }

    pub fn pop(&mut self) {
        let ev = self.events.pop();
        let time = Instant::now();
        ev.fame_duration = time - ev.start_time;
        ev.table[ev.table_index as usize] = ev.fame_duration.as_micros() as u32;

        ev.table_index += 1;
        if ev.table_index as usize >= ev.table.len() {
            ev.table_index = 0;
        }
    }

    pub fn draw(&mut self) -> String {
        let mut result = std::mem::take(&mut self.out);
        result.push_str("                         \n");
        result.push_str("   MIN    MAX    AVG     \n");
        result.push_str("                         \n");

        let mut depth = 0;
        for e in self.events.prev_items.iter() {
            match e {
                Item::Push(name, ev) => {
                    let mut t_min = 1_000_000_000;
                    let mut t_max = 0;
                    let mut t_avg = 0;

                    for e in ev.table.iter().copied() {
                        t_min = t_min.min(e);
                        t_max = t_max.max(e);
                        t_avg += e as u64;
                    }

                    t_avg /= ev.table.len() as u64;

                    result.push_str(&format!("{:6} {:6} {:6} µs | ", t_min, t_max, t_avg,));
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
