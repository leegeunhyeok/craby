fn main() {
  cxx_build::bridge("src/ffi.rs")
      .std("c++17")
      .compile("cxxbridge-demo");

  println!("cargo:rerun-if-changed=include/ffi.h");
}
