struct Stuff {
    strings: Vec<String>
}

impl Stuff {
    pub fn new() -> Self { Stuff { strings: Vec::new() } }

    pub fn last(&self) -> Option<&str> {
        Some(&self.strings[self.strings.len() - 1])
    }

    pub fn push(&mut self, s: &str) {
        self.strings.push(s.to_string());
    }
}

fn main() {
    let mut stuff = Stuff::new();
    let mut last = None;
    println!("{:?}, {:?}", stuff.strings, last);
    stuff.push("Hello");
    last = stuff.last();
    println!("{:?}, {:?}", stuff.strings, last);
    stuff.push("World");
    last = stuff.last();
    println!("{:?}, {:?}", stuff.strings, last);
}
