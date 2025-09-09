use std::path::PathBuf;

pub fn crate_dir(project_root: &PathBuf, crate_name: &str) -> PathBuf {
    project_root.join("crates").join(crate_name)
}

pub fn crate_manifest_path(project_root: &PathBuf, crate_name: &str) -> PathBuf {
    crate_dir(project_root, crate_name).join("Cargo.toml")
}
