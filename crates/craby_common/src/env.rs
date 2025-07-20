use std::{path::Path, process::Command};

use log::debug;

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

pub fn is_cargo_ndk_installed() -> bool {
    match std::process::Command::new("cargo")
        .args(["ndk", "--version"])
        .status()
    {
        Ok(status) => status.success(),
        Err(e) => {
            debug!("cargo-ndk not installed: {}", e);
            false
        }
    }
}

pub fn is_xcode_installed() -> bool {
    match std::process::Command::new("xcodebuild")
        .arg("-version")
        .status()
    {
        Ok(status) => status.success(),
        Err(e) => {
            debug!("xcodebuild not installed: {}", e);
            false
        }
    }
}

pub fn get_installed_targets() -> Result<Vec<String>, anyhow::Error> {
    let output = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()?;

    if !output.status.success() {
        debug!("Reason: {}", String::from_utf8_lossy(&output.stderr));
        return Err(anyhow::anyhow!("Failed to get installed targets"));
    }

    let targets = String::from_utf8(output.stdout)?;
    let targets = targets
        .lines()
        .map(|line| line.trim().to_string())
        .collect();

    Ok(targets)
}

#[allow(non_camel_case_types)]
pub enum Platform {
    Android,
    iOS,
}
