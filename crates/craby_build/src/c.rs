use std::{
    fs,
    path::{Path, PathBuf},
};

use craby_common::utils::{to_header_name, SanitizedString};
use log::{debug, info};

use crate::utils::{binding_header_dir, crate_dir};

pub fn generate_c_bindings(
    project_root: &Path,
    lib_name: &SanitizedString,
) -> Result<PathBuf, anyhow::Error> {
    let lib_crate_path = crate_dir(&project_root.to_path_buf(), "lib");
    let header_dir = binding_header_dir(&project_root.to_path_buf());
    let header_path = header_dir.join(to_header_name(lib_name));

    clean_binding_headers(&project_root.to_path_buf())?;

    let bindings = cbindgen::generate(lib_crate_path)?;
    let written = bindings.write_to_file(&header_path);
    debug!("C bindings written to {}", header_path.display());

    if !written {
        info!("C bindings are up to date");
    }

    Ok(header_path)
}

fn clean_binding_headers(project_root: &PathBuf) -> Result<(), anyhow::Error> {
    let header_dir = binding_header_dir(project_root);
    let files = fs::read_dir(header_dir)?;

    for file in files {
        let file = file?;
        if file.file_name().to_str().unwrap().ends_with(".h") {
            debug!("Removing existing header file {}", file.path().display());
            fs::remove_file(file.path())?;
        }
    }

    Ok(())
}
