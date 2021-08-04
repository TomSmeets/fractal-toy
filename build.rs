use includedir_codegen::Compression;

fn main() {
    includedir_codegen::start("STATIC_RES_FILES")
        .dir("res", Compression::Gzip)
        .build("static_res_files.rs")
        .unwrap();
}
