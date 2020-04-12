use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

mod connection;
use self::connection::Connection;

pub fn hexdump(xs: &[u8]) {
    if xs.is_empty() {
        return;
    }

    for chunck in xs.chunks(32) {
        for &b in chunck {
            print!("{:02x} ", b);
        }
        for &b in chunck {
            let mut c = char::from(b);
            if !c.is_alphanumeric() {
                c = '.';
            }
            print!("{}", c);
        }
        println!();
    }
}

fn main() {
    let mut srv = Connection::server("127.0.0.1:25566");
    let mut out = Connection::client("127.0.0.1:25565");

    let mut buffer = vec![0; 1024 * 1024];
    loop {
        if let Some(n) = srv.read(&mut buffer) {
            let buffer = &mut buffer[0..n];
            println!("loop: cli -> srv");
            hexdump(&buffer);
            out.write(&buffer);
            println!("Done");
        }

        if let Some(n) = out.read(&mut buffer) {
            let buffer = &mut buffer[0..n];
            println!("loop: cli <- srv");
            hexdump(&buffer);
            srv.write(&buffer);
            println!("Done");
        }

        std::thread::sleep_ms(10);
    }

    /*

    loop {
        let n = out.read(&mut bytes).unwrap();
        for b in bytes.iter().take(n) {
            print!("{:2x}", b);
        }
        println!();
    }
    */
}
