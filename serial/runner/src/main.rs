use std::env;
use std::ffi::c_void;
use std::ffi::OsStr;
use std::fs;
use std::path::*;

extern crate libloading as lib;
use libloading::os::unix::*;

struct ReloadLib {
    #[allow(dead_code)]
    lib: Library,
    f_init: Symbol<unsafe extern "C" fn() -> *mut c_void>,
    f_update: Symbol<unsafe extern "C" fn(*mut c_void) -> bool>,
}

impl ReloadLib {
    fn new(path: &OsStr) -> Self {
        unsafe {
            let lib = Library::open(Some(path), 1).unwrap();
            let f_init = lib.get(b"prog_init").unwrap();
            let f_update = lib.get(b"prog_update").unwrap();

            ReloadLib {
                f_init,
                f_update,
                lib,
            }
        }
    }

    fn init(&mut self) -> *mut c_void {
        unsafe { (self.f_init)() }
    }

    fn update(&mut self, s: *mut c_void) -> bool {
        unsafe { (self.f_update)(s) }
    }
}

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

    let mut lib = ReloadLib::new(lib_path.as_os_str());
    let s = lib.init();

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

            let tmp_path = {
                let mut buf = PathBuf::new();
                buf.push(&tmp);
                buf.push(format!("tmp_{}.so", i));
                buf
            };

            fs::copy(&lib_path, &tmp_path).unwrap();
            lib = ReloadLib::new(tmp_path.as_os_str());
            // fs::remove_file(tmp_path).unwrap();
            i += 1;

            println!("reload");
            // s.reload();
        }

        quit = lib.update(s);
    }
}
