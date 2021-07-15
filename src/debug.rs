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

    last_frame_events: Vec<DebugEvent>,
    current_frame_events: Vec<DebugEvent>,
}

impl Debug {
    pub fn new() -> Self {
        Debug {
            info: String::new(),
            out: String::new(),

            last_frame_events: Vec::new(),
            current_frame_events: Vec::new(),
        }
    }

    pub fn begin(&mut self) {
        std::mem::swap(&mut self.current_frame_events, &mut self.last_frame_events);
        self.current_frame_events.clear();

        std::mem::swap(&mut self.out, &mut self.info);
        self.info.clear();
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
}
