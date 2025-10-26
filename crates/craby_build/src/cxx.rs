pub fn setup() {
    cxx_build::bridge("src/ffi.rs")
        .std("c++20")
        .include("include")
        .compile("cxxbridge")
}
