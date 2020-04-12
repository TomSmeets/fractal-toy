use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

pub struct Connection {
    conn: TcpStream,
}

impl Connection {
    pub fn server(addr: &str) -> Self {
        let srv = TcpListener::bind(addr).unwrap();
        let srv = srv.incoming().next().unwrap().unwrap();
        Connection::from(srv)
    }

    pub fn client(addr: &str) -> Self {
        let out = TcpStream::connect("127.0.0.1:25565").unwrap();
        Connection::from(out)
    }

    pub fn from(conn: TcpStream) -> Self {
        conn.set_nonblocking(true).unwrap();
        Connection { conn }
    }

    pub fn write(&mut self, data: &[u8]) {
        self.conn.write_all(data).unwrap();
    }

    pub fn read(&mut self, data: &mut [u8]) -> Option<usize> {
        match self.conn.read(data) {
            Ok(0) => None,
            Ok(n) => Some(n),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    None
                } else {
                    panic!("{:?}", e)
                }
            }
        }
    }
}
