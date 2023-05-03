fn main() {
    cbindgen::Builder::new()
        .with_crate(".")
        .with_header(
            "typedef void Void;\ntemplate<typename T>\nusing Opaque=void;\ntypedef uintptr_t AtomicUsize;".to_owned(),
        )
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("crust_std.h");
}
