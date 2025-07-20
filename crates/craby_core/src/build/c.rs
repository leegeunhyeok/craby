use std::path::{Path, PathBuf};

use craby_common::utils::{
    fs::clean_binding_headers,
    path::{binding_header_dir, crate_dir},
};
use log::{debug, info};

pub fn generate_c_bindings(project_root: &Path, lib_name: &str) -> Result<PathBuf, anyhow::Error> {
    let lib_crate_path = crate_dir(&project_root.to_path_buf(), "ios");
    let header_dir = binding_header_dir(&project_root.to_path_buf());
    let header_path = header_dir.join(format!("lib{}.h", lib_name));

    clean_binding_headers(&project_root.to_path_buf())?;

    let bindings = cbindgen::generate(lib_crate_path)?;
    let written = bindings.write_to_file(&header_path);
    debug!("C bindings written to {}", header_path.display());

    if !written {
        info!("C bindings are up to date");
    }

    Ok(header_path)
}
