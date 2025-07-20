use std::{
    fs,
    path::{Path, PathBuf},
};

use craby_common::constants::TEMP_DIR;
use log::{debug, info};

pub fn generate_c_bindings(project_root: &Path, lib_name: &str) -> Result<PathBuf, anyhow::Error> {
    let tmp_dir = project_root.join(TEMP_DIR);
    let lib_crate_path = project_root.join("crates").join("ios");
    let header_dir = tmp_dir.join("include");
    let header_path = header_dir.join(format!("lib{}.h", lib_name));

    let files = fs::read_dir(header_dir)?;
    for file in files {
        let file = file?;
        if file.file_name().to_str().unwrap().ends_with(".h") {
            debug!("Removing existing header file {}", file.path().display());
            fs::remove_file(file.path())?;
        }
    }

    let bindings = cbindgen::generate(lib_crate_path)?;
    let written = bindings.write_to_file(&header_path);
    debug!("C bindings written to {}", header_path.display());

    if !written {
        info!("C bindings are up to date");
    }

    Ok(header_path)
}
