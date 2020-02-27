use std::io::{self, BufRead, Read, Write};

pub fn main() {
    println!("before");
    my_very_unique_function();
    println!("after");
}


pub fn my_very_unique_function() {
    let a = 1;
    let b = 2;
    println!("a + b = {}", a + b);

    let mut s1 = String::from("Hello");
    let mut s2 = String::from("World");
    println!("s1: {}", s1);
    println!("s2: {}", s2);

    s1 += &s2;
    println!("s1: {}", s1);
    println!("s2: {}", s2);
}
