extern crate gl_generator;

use gl_generator::{Api, Fallbacks, GlobalGenerator, Profile, Registry, StructGenerator};
use std::env;
use std::fs::File;
use std::path::Path;

fn main() {
    let dest = env::var("OUT_DIR").unwrap();

    println!("cargo:rerun-if-changed=build.rs");

    let path = Path::new(&dest).join("bindings.rs");

    let mut file = File::create(&path).unwrap();
    // StructGenerator produces much smaller 'bindings.rs'
    Registry::new(Api::Gl, (3, 3), Profile::Core, Fallbacks::None, [])
        .write_bindings(StructGenerator, &mut file)
        .unwrap();
}
