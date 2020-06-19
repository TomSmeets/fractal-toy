use crate::Id;

#[derive(Default)]
pub struct UIStack {
    current: Id,
    stack: Vec<Id>,
}

impl UIStack {
    pub fn new() -> Self {
        UIStack {
            current: Id::root(),
            stack: Vec::new(),
        }
    }

    // TODO: this should not be needed,
    pub fn clear(&mut self) {
        self.current = Id::root();
        self.stack.clear();
    }

    pub fn depth(&self) -> u32 {
        self.stack.len() as u32
    }

    /// Returns current active id
    pub fn id(&self) -> Id {
        self.current
    }

    pub fn begin_raw(&mut self, name: &[u8]) -> Id {
        self.stack.push(self.current);
        self.current = Id::from_bytes(name, self.current);
        self.current
    }

    pub fn begin(&mut self, name: &str) -> Id {
        self.begin_raw(name.as_bytes())
    }

    pub fn end(&mut self) {
        // TODO: handle to many ends?
        self.current = self.stack.pop().unwrap();
    }
}
