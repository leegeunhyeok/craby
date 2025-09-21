use std::path::PathBuf;

use craby_common::config::load_config;

pub fn build_setup(project_root: &PathBuf) {
    let res = load_config(project_root);

    // FIXME: To be used later
    let _config = match res {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Load config failed: {}", e);
            std::process::exit(1);
        }
    };

    // TODO
    // See: https://github.com/dtolnay/cxx/tree/master/demo
    cxx_build::bridge("src/ffi.rs")
        .std("c++20")
        .compile("cxxbridge");

    println!("cargo:rerun-if-changed=src/ffi.rs");
}
