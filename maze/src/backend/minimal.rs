use crate::core::Generator;
use crate::core::Maze;

#[allow(dead_code)]
pub fn run() {
    let mut maze = Maze::new(53, 53);
    let mut gen = Generator::new();
    while let Some(p) = gen.next(&mut maze) {
        println!("gen.next: {:?}", p);
    }
    println!("done");
}
