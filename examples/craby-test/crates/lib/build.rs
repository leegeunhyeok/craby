fn main() {
    cxx_build::bridge("src/ffi.rs")
        .std("c++20")
        .include("include")
        .compile("cxxbridge");

    println!("cargo:rerun-if-changed=include/EventEmitter.h");
}
