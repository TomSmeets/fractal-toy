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

#[derive(Debug)]
struct Texture {
    text: String,
}

impl Drop for Texture {
    fn drop(&mut self) {
        println!("bye bye Texture");
    }
}



fn main() {
    let r = Texture { text: String::from("Hello world") };
    let r = Rc::new(r);
    let w1 = Rc::downgrade(&r);
    let w2 = Rc::downgrade(&r);
    println!("r: strong={} weak={}", Rc::strong_count(&r), Rc::weak_count(&r));
    println!("r: {:?}", r);
    println!("r: strong={} weak={}", Weak::strong_count(&w1), Weak::weak_count(&w1));
    println!("r: strong={} weak={}", Weak::strong_count(&w2), Weak::weak_count(&w2));
}
