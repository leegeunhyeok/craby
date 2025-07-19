use std::path::Path;

pub fn is_rustup_installed() -> bool {
    std::process::Command::new("rustup")
        .arg("--version")
        .output()
        .is_ok()
}

pub fn is_initialized(project_root: &Path) -> bool {
    let crates_dir = project_root.join("crates");

    project_root.join(".craby").exists()
        && project_root.join("Cargo.toml").exists()
        && crates_dir.join("lib").join("Cargo.toml").exists()
        && crates_dir.join("android").join("Cargo.toml").exists()
        && crates_dir.join("iOS").join("Cargo.toml").exists()
}

#[allow(non_camel_case_types)]
pub enum Platform {
    Android,
    iOS,
}
