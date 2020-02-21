extern crate serial;

use std::env;
use std::fs;
use std::path::*;

use serial::game::*;

extern crate libloading as lib;
use libloading::os::unix::*;

fn main() {
    let exe = env::current_exe().unwrap();
    let dir = exe.parent().unwrap();

    let lib_path = {
        let mut buf = PathBuf::new();
        buf.push(dir);
        buf.push("lib.so");
        buf
    };

    println!("self: {:?}", lib_path);


    let mut lib = Library::open(Some(&lib_path), 1).unwrap();

    let mut s : State = State::new();


    let mut i = 0;
    let mut quit = false;
    // let mut update: Option< &Symbol<unsafe extern fn(*mut State) -> bool> > = None;
    let mut t_old = std::time::UNIX_EPOCH;
    while !quit {
        let t_new = lib_path.metadata().unwrap().modified().unwrap();
        if t_new != t_old {
            println!("reload! {:?}", t_new);
            t_old = t_new;

            drop(lib);

            let tmp_path = {
                let mut buf = PathBuf::new();
                buf.push(dir);
                buf.push(format!("tmp_{}.so", i));
                buf
            };

            fs::copy(&lib_path, &tmp_path).unwrap();
            lib = Library::open(Some(&tmp_path), 1).unwrap();
            fs::remove_file(tmp_path).unwrap();
            i += 1;
        }

        unsafe {
            let l_update : Symbol<unsafe extern fn(*mut State) -> bool> = lib.get(b"prog_update").unwrap();
            quit = l_update(&mut s);
        }
    }
}
