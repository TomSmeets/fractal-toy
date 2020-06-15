use crate::Id;
use std::collections::BTreeMap;

pub enum RenderCommand {
    Active(bool),
    Other(bool),
    Text(u32, String),
}

#[derive(Default)]
struct ElementData {
    open: bool,
}

pub enum Nav {
    Up,
    Down,
    Left,
    Right,
    Close,
}

#[derive(Default)]
pub struct U0 {
    stack: Vec<Id>,
    active: Option<Id>,
}

impl U0 {
    fn clear(&mut self) {
        self.stack.clear();
    }

    fn is_active(&self) -> bool {
        self.active.is_some() && self.id() == self.active
    }

    fn id(&self) -> Option<Id> {
        self.stack.last().map(|x| *x)
    }

    fn begin(&mut self, name: &str) {
        let id = Id::new(name, self.id().unwrap_or(Id::root()));
        self.stack.push(id);

        if self.active == None {
            self.active = Some(id);
        }
    }

    fn end(&mut self) {
        let _ = self.stack.pop().unwrap();
    }
}

#[derive(Default)]
pub struct UI {
    ui: U0,
    nav: Navigation,

    data: BTreeMap<Id, ElementData>,
    draw: Vec<RenderCommand>,
}

enum Operation {
    Push(Id),
    Pop,
}

#[derive(Default)]
pub struct Navigation {
    active: usize,
    path: Vec<Operation>,
}

impl Navigation {
    pub fn clear(&mut self) {
        self.active = 0;
        self.path = Vec::new();
    }

    pub fn begin(&mut self, ui: &U0) {
        let id = ui.id().unwrap();

        if ui.is_active() {
            self.active = self.path.len();
        }

        self.path.push(Operation::Push(id));
    }

    pub fn end(&mut self) {
        self.path.push(Operation::Pop);
    }

    pub fn child(&self) -> Option<Id> {
        match self.path.get(self.active + 1) {
            Some(Operation::Push(i)) => Some(*i),
            _ => None,
        }
    }

    pub fn next(&self) -> Option<Id> {
        let mut it = self.path[self.active..].iter();
        let mut depth = 0;
        while let Some(i) = it.next() {
            match i {
                Operation::Push(_) => depth += 1,
                Operation::Pop => depth -= 1,
            }
            if depth == 0 {
                break;
            }
        }

        match it.next()? {
            Operation::Push(id) => Some(*id),
            Operation::Pop => None,
        }
    }

    pub fn prev(&self) -> Option<Id> {
        let mut it = self.path[0..self.active].iter().rev();
        let mut depth = 0;
        while let Some(i) = it.next() {
            match i {
                Operation::Push(_) => depth -= 1,
                Operation::Pop => depth += 1,
            }

            if depth == 0 {
                return match i {
                    Operation::Push(id) => Some(*id),
                    Operation::Pop => None,
                };
            }
        }

        return None;
    }

    pub fn parent(&self) -> Option<Id> {
        let mut it = self.path[0..self.active].iter().rev();
        let mut depth = 0;
        while let Some(i) = it.next() {
            match i {
                Operation::Push(_) => depth -= 1,
                Operation::Pop => depth += 1,
            }

            if depth == -1 {
                return match i {
                    Operation::Push(id) => Some(*id),
                    Operation::Pop => None,
                };
            }
        }

        return None;
    }
}

impl UI {
    pub fn new() -> Self {
        UI::default()
    }

    pub fn clear(&mut self) {
        self.ui.clear();
        self.nav.clear();
    }

    pub fn draw(&mut self) -> Vec<RenderCommand> {
        self.draw.drain(..).collect()
    }

    pub fn current_id(&self) -> Id {
        self.ui.id().unwrap_or(Id::root())
    }

    pub fn begin(&mut self, n: &str) -> bool {
        self.beg(n);

        let id = self.current_id();
        let active = self.ui.is_active();
        let indent = self.ui.stack.len() as u32;

        if active {
            self.draw.push(RenderCommand::Active(true));
            self.draw.push(RenderCommand::Other(true));
        }

        let mut cmd = String::new();

        let data = self.data.entry(id).or_default();
        let result = if data.open {
            cmd.push_str(" ");
            cmd.push_str(n);
            cmd.push_str(":");
            true
        } else {
            cmd.push_str(" ");
            cmd.push_str(n);
            cmd.push_str(" (..)");
            false
        };

        self.draw.push(RenderCommand::Text(indent, cmd));

        if active {
            self.draw.push(RenderCommand::Active(false));
        }

        if !result {
            self.end();
        }

        result
    }

    pub fn beg(&mut self, n: &str) {
        self.ui.begin(n);

        self.nav.begin(&self.ui);
    }

    pub fn end(&mut self) {
        if self.ui.is_active() {
            self.draw.push(RenderCommand::Other(false));
        }

        self.nav.end();
        self.ui.end();
    }

    pub fn do_nav(&mut self, nav: Nav) {
        if let Some(mut data) = self.data.get_mut(&self.ui.active.unwrap()) {
            match nav {
                Nav::Right => {
                    if !data.open {
                        data.open = true;
                        return;
                    }
                },
                Nav::Close => data.open = !data.open,
                _ => (),
            }
        }

        if let Some(n) = match nav {
            Nav::Up => self.nav.prev(),
            Nav::Down => self.nav.next(),
            Nav::Left => self.nav.parent(),
            Nav::Right => self.nav.child(),

            _ => None,
        } {
            self.ui.active = Some(n);
        }
    }
}
