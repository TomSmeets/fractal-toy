mod backend;
mod core;

// core should not depend on any platform spesific stuff.
// core is like a library. it should do as much as possible.
// such that the platform spesific can be as minimal as possible

fn main() {
    self::backend::run();
}
