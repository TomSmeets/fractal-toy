use std::rc::{Weak, Rc};

#[derive(Debug)]
struct Stuff {
    current: Weak<String>,
    strings: Vec<Rc<String>>,
}

impl Stuff {
    pub fn new() -> Self {
        let strings = vec![];
        Stuff { strings, current: Weak::new() }
    }

    pub fn last(&self) -> Option<Rc<String>> {
        self.current.upgrade()
    }

    pub fn push(&mut self, s: &str) {
        let s = Rc::new(s.to_string());
        self.current = Rc::downgrade(&s);
        self.strings.push(s);
    }
}

fn main() {
    let mut stuff = Stuff::new();
    println!("{:?}, {:?}", stuff.strings, stuff.last());
    stuff.push("Hello");
    println!("{:?}, {:?}", stuff.strings, stuff.last());
    stuff.push("World");
    println!("{:?}, {:?}", stuff.strings, stuff.last());
}
