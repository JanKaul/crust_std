fn main() {
    cbindgen::generate(".")
        .expect("Unable to generate bindings")
        .write_to_file("crust_std.h");
}
