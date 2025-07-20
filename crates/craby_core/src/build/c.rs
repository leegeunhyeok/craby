use std::path::{Path, PathBuf};

use log::{debug, info};

pub fn generate_c_bindings(project_root: &Path, lib_name: &str) -> Result<PathBuf, anyhow::Error> {
    let tmp_dir = project_root.join(".craby");
    let lib_crate_path = project_root.join("crates").join("ios");
    let header_path = tmp_dir.join("include").join(format!("lib{}.h", lib_name));

    let bindings = cbindgen::generate(lib_crate_path)?;
    let written = bindings.write_to_file(&header_path);
    debug!("C bindings written to {}", header_path.display());

    if !written {
        info!("C bindings are up to date");
    }

    Ok(header_path)
}
