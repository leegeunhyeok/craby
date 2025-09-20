fn main() {
    cxx_build::bridge("src/ffi.rs")
        .std("c++17")
        .compile("cxxbridge");

    println!("cargo:rerun-if-changed=src/ffi.h");
}
