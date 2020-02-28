use std::env;
use std::ffi::c_void;
use std::fs;
use std::path::*;

extern crate libloading as lib;
use libloading::os::unix::*;

fn main() {
    let exe = env::current_exe().unwrap();
    let dir = exe.parent().unwrap();

    let tmp = {
        let mut d = env::temp_dir();
        let pid = std::process::id();
        d.push(format!("dynamic-reload-at-{}", pid));
        fs::create_dir_all(&d).unwrap();
        d
    };

    let lib_path = {
        let mut buf = PathBuf::new();
        buf.push(dir);
        buf.push("lib.so");
        buf
    };

    println!("self: {:?}", lib_path);

    let mut lib = Library::open(Some(&lib_path), 1).unwrap();

    let s: *mut c_void = unsafe {
        let l_init: Symbol<unsafe extern "C" fn() -> *mut c_void> = lib.get(b"prog_init").unwrap();
        l_init()
    };

    println!("init");

    let mut i = 0;
    let mut quit = false;
    // let mut update: Option< &Symbol<unsafe extern fn(*mut State) -> bool> > = None;
    let mut t_old = std::time::UNIX_EPOCH;
    while !quit {
        let t_new = lib_path.metadata().unwrap().modified().unwrap();
        if t_new != t_old {
            println!("unload");
            // s.unload();

            println!("reload! {:?}", t_new);
            t_old = t_new;

            drop(lib);

            let tmp_path = {
                let mut buf = PathBuf::new();
                buf.push(&tmp);
                buf.push(format!("tmp_{}.so", i));
                buf
            };

            fs::copy(&lib_path, &tmp_path).unwrap();
            lib = Library::open(Some(&tmp_path), 1).unwrap();
            // fs::remove_file(tmp_path).unwrap();
            i += 1;

            println!("reload");
            // s.reload();
        }

        unsafe {
            let l_update: Symbol<unsafe extern "C" fn(*mut c_void) -> bool> =
                lib.get(b"prog_update").unwrap();
            quit = l_update(s);
        }
    }
}
