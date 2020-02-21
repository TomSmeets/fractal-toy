use std::io::{self, BufRead, Read, Write};

pub fn main() {
    let stdin  = io::stdin();
    let mut stdout = io::stdout();
    let mut handle = stdin.lock();

    loop {
        let mut buffer = String::new();
        print!("> ");
        stdout.flush();
        match handle.read_line(&mut buffer) {
            Ok(n) => {
                if n == 0 {
                    break;
                }
                println!("n = {}", n);
                print!("{}", buffer);
            }
            Err(e) => {
                println!("error: {}", e);
                break;
            }
        }
    }
}

