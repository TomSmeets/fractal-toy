use gl_generator::*;
use std::env;
use std::fs::File;
use std::path::Path;

fn main() {
    let dest = env::var("OUT_DIR").unwrap();

    println!("cargo:rerun-if-changed=build.rs");

    let path = Path::new(&dest).join("bindings-gl.rs");

    let mut file = File::create(&path).unwrap();

    // StructGenerator produces much smaller 'bindings.rs' file
    Registry::new(Api::Gl, (3, 3), Profile::Core, Fallbacks::None, [])
        .write_bindings(StructGenerator, &mut file)
        .unwrap();
}
