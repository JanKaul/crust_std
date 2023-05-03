fn main() {
    cbindgen::Builder::new()
        .with_crate(".")
        .with_namespace("crust_std")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("crust_std.h");
}
