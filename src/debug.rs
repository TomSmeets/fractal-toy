use std::time::{Instant, SystemTime};
use std::collections::BTreeMap;

struct TimeEntry {
    dt_avrg: f32,
    dt_max:  f32,
}

pub struct Debug {
    info: String,
    out: String,

    time_str_next: String,
    time_str: String,
    time_name: String,
    time: Instant,


    time_data: BTreeMap<&'static str, TimeEntry>,
}

impl Debug {
    pub fn new() -> Self {
        Debug {
            info: String::new(),
            out: String::new(),

            time_data: BTreeMap::new(),
            time_str: String::new(),
            time_str_next: String::new(),
            time_name: "?".to_string(),
            time: Instant::now(),
        }
    }

    pub fn begin(&mut self) {
        std::mem::swap(&mut self.out, &mut self.info);
        std::mem::swap(&mut self.time_str, &mut self.time_str_next);
        self.info.clear();
        self.time_str_next.clear();
    }

    pub fn draw(&mut self) -> String {
        let mut result = std::mem::take(&mut self.out);
        result.push_str(&self.time_str);
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

        let mut e = self.time_data.entry(name).or_insert(TimeEntry { dt_avrg: 0.0, dt_max: 0.0, });

        e.dt_avrg = e.dt_avrg * 0.999 + dt as f32 * 0.001;
        e.dt_max  = (e.dt_max).max(dt);

        self.time_str_next.push_str(&format!("{:6} max, {:6} avg, {}\n", e.dt_max.round(), e.dt_avrg.round(), self.time_name));

        self.time = time;
        self.time_name.clear();
        self.time_name.push_str(name);

    }
}
