use std::path::Path;

use log::info;

pub fn generate_c_bindings(project_root: &Path, lib_name: &str) -> Result<(), anyhow::Error> {
    let tmp_dir = project_root.join(".craby");
    let lib_crate_path = project_root.join("crates").join("lib");
    let header_path = tmp_dir.join("include").join(format!("lib{}.h", lib_name));

    let bindings = cbindgen::generate(lib_crate_path)?;
    let written = bindings.write_to_file(header_path);

    if written {
        info!("Generated C bindings for {}", lib_name);
    } else {
        info!("C Bindings are up to date");
    }

    Ok(())
}
