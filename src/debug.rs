pub struct Debug {
    info: String,
}

impl Debug {
    pub fn new() -> Self {
        Debug {
            info: String::new()
        }
    }

    pub fn begin(&mut self) {
        self.info.clear();
    }

    pub fn draw(&mut self) -> &str {
        &self.info
    }

    pub fn print(&mut self, s: &str) {
        self.info.push_str(s);
        self.info.push_str("\n");
    }
}
