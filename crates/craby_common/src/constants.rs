use std::path::PathBuf;

use crate::utils::string::SanitizedString;

pub mod toolchain {
    pub const TARGETS: &[&str] = &[
        // Android
        "aarch64-linux-android",
        "armv7-linux-androideabi",
        "x86_64-linux-android",
        "i686-linux-android",
        // iOS
        "aarch64-apple-ios",
        "aarch64-apple-ios-sim",
    ];
}

pub mod android {
    pub const ABI_TARGETS: &[&str] = &[
        // Target: aarch64-linux-android
        "arm64-v8a",
        // Target: armv7-linux-androideabi
        "armeabi-v7a",
        // Target: x86_64-linux-android
        "x86_64",
        // Target: i686-linux-android
        "x86",
    ];
}

pub mod ios {}

pub const GENERATED_MOD: &str = "generated";
pub const TEMP_DIR: &str = ".craby";

/// Returns the name of the library for the Cargo manifest.
///
/// Cargo library names cannot contain hyphens,
/// so we use flat case for the library name in the Cargo manifest.
pub fn cargo_lib_name(name: &SanitizedString) -> String {
    format!("lib{}.a", name.0.replace("_", ""))
}

pub fn lib_name(name: &SanitizedString) -> String {
    format!("lib{}-craby.a", name.0.replace("_", ""))
}

pub fn lib_header_name(name: &SanitizedString) -> String {
    format!("lib{}-craby.h", name.0.replace("_", ""))
}

pub fn impl_mod_name(name: &SanitizedString) -> String {
    format!("{}_impl", name.0)
}

pub fn tmp_dir(project_root: &PathBuf) -> PathBuf {
    project_root.join(TEMP_DIR)
}

pub fn crate_target_dir(project_root: &PathBuf, target: &String) -> PathBuf {
    project_root.join("target").join(target).join("release")
}

pub fn crate_dir(project_root: &PathBuf) -> PathBuf {
    project_root.join("crates").join("lib")
}

pub fn crate_manifest_path(project_root: &PathBuf) -> PathBuf {
    crate_dir(project_root).join("Cargo.toml")
}

pub fn binding_header_dir(project_root: &PathBuf) -> PathBuf {
    tmp_dir(project_root).join("include")
}
