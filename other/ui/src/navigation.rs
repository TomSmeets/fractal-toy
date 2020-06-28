use crate::Id;
use crate::UIStack;

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

    pub fn begin(&mut self, stack: &UIStack, active: Option<Id>) {
        if Some(stack.id()) == active {
            self.active = self.path.len();
        }

        self.path.push(Operation::Push(stack.id()));
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
        let it = self.path[0..self.active].iter().rev();
        let mut depth = 0;
        for i in it {
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

        None
    }

    pub fn parent(&self) -> Option<Id> {
        let it = self.path[0..self.active].iter().rev();
        let mut depth = 0;
        for i in it {
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

        None
    }
}
