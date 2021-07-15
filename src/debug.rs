use std::collections::BTreeMap;
use std::time::Instant;

struct TimeEntry {
    dt_avrg: f32,
    dt_max: f32,
}

#[derive(Debug, Clone, Copy)]
enum DebugEvent {
    Push(&'static str, Instant),
    Pop(Instant),
}

pub struct Debug {
    info: String,
    out: String,

    time_str_next: String,
    time_str: String,
    time_name: &'static str,
    time: Instant,

    reset_time: u32,

    time_data: BTreeMap<&'static str, TimeEntry>,

    last_frame_events: Vec<DebugEvent>,
    current_frame_events: Vec<DebugEvent>,
}

impl Debug {
    pub fn new() -> Self {
        Debug {
            info: String::new(),
            out: String::new(),

            time_data: BTreeMap::new(),
            time_str: String::new(),
            time_str_next: String::new(),
            time_name: "?",
            time: Instant::now(),
            reset_time: 0,

            last_frame_events: Vec::new(),
            current_frame_events: Vec::new(),
        }
    }

    pub fn begin(&mut self) {
        std::mem::swap(&mut self.current_frame_events, &mut self.last_frame_events);
        self.current_frame_events.clear();

        std::mem::swap(&mut self.out, &mut self.info);
        self.info.clear();

        std::mem::swap(&mut self.time_str, &mut self.time_str_next);
        self.time_str_next.clear();

        self.reset_time += 1;

        // TODO: this can be improved
        // fter 10 seconds reset
        if self.reset_time > 180 * 10 {
            for e in self.time_data.values_mut() {
                e.dt_max = 0.0;
            }
            self.reset_time = 0;
        }
    }

    pub fn push(&mut self, name: &'static str) {
        self.current_frame_events
            .push(DebugEvent::Push(name, Instant::now()));
    }

    pub fn pop(&mut self) {
        self.current_frame_events
            .push(DebugEvent::Pop(Instant::now()));
    }

    pub fn draw(&mut self) -> String {
        let mut result = std::mem::take(&mut self.out);
        result.push_str(&self.time_str);

        // 0 1 2 3 2 3 2 1 2 1 0
        //  ( ( ( ) ( ) ) ( ) )
        fn find_pop(ev: &[DebugEvent]) -> Instant {
            let mut depth = 0;
            for e in ev {
                match e {
                    DebugEvent::Push(_, _) => depth += 1,
                    DebugEvent::Pop(t) => {
                        depth -= 1;

                        if depth == 0 {
                            return *t;
                        }
                    }
                }
            }

            panic!("Oh no")
        }

        result.push_str("-----------\n");

        let mut depth = 0;
        for i in 0..self.last_frame_events.len() {
            let e = self.last_frame_events[i];

            match e {
                DebugEvent::Push(name, start_time) => {
                    let end_time = find_pop(&self.last_frame_events[i..]);

                    result.push_str(&format!(
                        "{:6.2} µs | ",
                        (end_time - start_time).as_micros()
                    ));
                    for _ in 0..depth {
                        result.push_str("  ");
                    }
                    result.push_str(name);
                    result.push_str("\n");

                    depth += 1;
                }

                DebugEvent::Pop(time) => {
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

    pub fn time(&mut self, name: &'static str) {
        let time = Instant::now();
        let dt = time - self.time;
        let dt = dt.as_micros() as u32;
        let dt = dt as f32;

        let mut e = self.time_data.entry(self.time_name).or_insert(TimeEntry {
            dt_avrg: 0.0,
            dt_max: 0.0,
        });
        e.dt_avrg = e.dt_avrg * 0.999 + dt as f32 * 0.001;
        e.dt_max = (e.dt_max).max(dt);

        self.time_str_next.push_str(&format!(
            "{:6} max, {:6} avg, {}\n",
            e.dt_max.round(),
            e.dt_avrg.round(),
            self.time_name
        ));

        self.time = time;
        self.time_name = name;
    }
}
