use crate::Id;
use crate::Navigation;
use crate::UIStack;
use std::collections::BTreeMap;

pub enum RenderCommand {
    Active(bool),
    Other(bool),
    Text(u32, String),
}

#[derive(Default)]
pub struct ElementData {
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
pub struct UI {
    stack: UIStack,
    nav: Navigation,

    pub data: BTreeMap<Id, ElementData>,
    draw: Vec<RenderCommand>,
}

impl UI {
    pub fn new() -> Self {
        UI::default()
    }

    pub fn clear(&mut self) {
        self.stack.clear();
        self.nav.clear();
    }

    pub fn draw(&mut self) -> Vec<RenderCommand> {
        self.draw.drain(..).collect()
    }

    pub fn current_id(&self) -> Id {
        self.stack.id().unwrap_or(Id::root())
    }

    pub fn begin(&mut self, n: &str) -> bool {
        self.beg(n);

        let id = self.current_id();
        let active = self.stack.is_active();
        let indent = self.stack.depth();

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
        self.stack.begin(n);

        self.nav.begin(&self.stack);
    }

    pub fn end(&mut self) {
        if self.stack.is_active() {
            self.draw.push(RenderCommand::Other(false));
        }

        self.nav.end();
        self.stack.end();
    }

    pub fn do_nav(&mut self, nav: Nav) {
        if let Some(mut data) = self.data.get_mut(&self.stack.active.unwrap()) {
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
            self.stack.active = Some(n);
        }
    }
}
