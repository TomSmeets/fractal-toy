use crate::Id;

#[derive(Default)]
pub struct UIStack {
    stack: Vec<Id>,
    pub active: Option<Id>,
}

impl UIStack {
    // TODO: this should not be needed,
    pub fn clear(&mut self) {
        self.stack.clear();
    }

    pub fn depth(&self) -> u32 {
        self.stack.len() as u32
    }

    pub fn is_active(&self) -> bool {
        self.active.is_some() && self.id() == self.active
    }

    pub fn id(&self) -> Option<Id> {
        self.stack.last().map(|x| *x)
    }

    pub fn begin(&mut self, name: &str) -> Id {
        let id = Id::new(name, self.id().unwrap_or(Id::root()));
        self.stack.push(id);

        if self.active == None {
            self.active = Some(id);
        }

        id
    }

    pub fn end(&mut self) {
        let _ = self.stack.pop().unwrap();
    }
}
