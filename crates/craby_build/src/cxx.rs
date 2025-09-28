use std::{fs, path::PathBuf};

use craby_common::config::load_config;

pub fn replace_cxx_header(signal_path: &PathBuf) -> Result<(), anyhow::Error> {
    let signals_h = fs::read_to_string(&signal_path)?;
    let signals_h = signals_h.replace("\"rust/cxx.h\"", "\"cxx.h\"");
    fs::write(&signal_path, signals_h)?;
    Ok(())
}

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
