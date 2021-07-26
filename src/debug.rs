use std::cell::RefCell;
use std::collections::BTreeMap;

use instant::Instant;

const SAMPLE_COUNT: usize = 180;

struct TimeEntry {
    dt_avrg: f32,
    dt_max: f32,
}

#[derive(Debug, Clone, Copy)]
struct DebugEvent {
    table_index: u32,
    calls: u64,
    table: [u32; SAMPLE_COUNT],
}

enum Item<T> {
    Push(&'static str, T),
    Pop,
}

pub struct Debug {
    info: String,
    out: String,

    stack: Vec<(&'static str, Instant)>,
    events: BTreeMap<&'static str, DebugEvent>,
}

thread_local! {
    static STACK: RefCell<Vec<(&'static str, Instant)>> = RefCell::new(Vec::new());
}

use std::sync::Mutex;

use ::lazy_static::lazy_static;
lazy_static! {
    static ref EVENTS: Mutex<BTreeMap<&'static str, DebugEvent>> = Mutex::new(BTreeMap::new());
}

impl Debug {
    pub fn new() -> Self {
        Debug {
            info: String::new(),
            out: String::new(),

            stack: Vec::new(),
            events: BTreeMap::new(),
        }
    }

    pub fn begin(&mut self) {
        std::mem::swap(&mut self.out, &mut self.info);
        self.info.clear();
    }

    pub fn push(name: &'static str) {
        STACK.with(|s| s.borrow_mut().push((name, Instant::now())));
    }

    pub fn pop() {
        let end_time = Instant::now();
        let (name, start_time) = STACK.with(|s| s.borrow_mut().pop().unwrap());

        let mut events = EVENTS.lock().unwrap();
        let ev = events.entry(name).or_insert_with(|| DebugEvent {
            table_index: 0,
            calls: 0,
            table: [0; SAMPLE_COUNT],
        });

        ev.table[ev.table_index as usize] = (end_time - start_time).as_micros() as u32;

        ev.table_index += 1;
        ev.calls += 1;
        if ev.table_index as usize >= ev.table.len() {
            ev.table_index = 0;
        }
    }

    pub fn draw(&mut self) -> String {
        let mut result = std::mem::take(&mut self.out);
        result.push_str("                                 \n");
        result.push_str("    MIN     MAX     AVG     Calls\n");
        result.push_str("                                 \n");

        let events = EVENTS.lock().unwrap();
        let mut table = events
            .iter()
            .map(|(name, ev)| {
                let mut t_min = 1_000_000_000;
                let mut t_max = 0;
                let mut t_avg = 0;

                for e in ev.table.iter().copied() {
                    t_min = t_min.min(e);
                    t_max = t_max.max(e);
                    t_avg += e as u64;
                }

                t_avg /= ev.table.len() as u64;

                (name, t_min, t_max, t_avg as u32, ev.calls)
            })
            .collect::<Vec<_>>();

        table.sort_by_key(|(_, _, _, a, _)| -(*a as i64));

        for (name, t_min, t_max, t_avg, calls) in table.into_iter() {
            result.push_str(&format!(
                "{:7} {:7} {:7} {:7} | {}\n",
                t_min, t_max, t_avg, calls, name
            ));
        }

        result
    }

    pub fn print(&mut self, s: &str) {
        self.info.push_str(s);
        self.info.push_str("\n");
    }
}
