pub struct Time {
    pub dt: f32,
    pub dt_inv: f32,
    pub iteration: i32,
    pub time: f32,
}

impl Time {
    pub fn new(dt: f32) -> Self {
        Time {
            dt,
            dt_inv: 1.0 / dt,
            iteration: 0,
            time: 0.0,
        }
    }

    pub fn update(&mut self) {
        self.iteration += 1;
        self.time += self.dt;
    }
}
