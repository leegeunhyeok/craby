use std::path::{Path, PathBuf};

use crate::utils::string::{flat_case, snake_case, SanitizedString};

pub const HASH_COMMAND_PREFIX: &str = "// Hash:";

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
        "x86_64-apple-ios",
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

pub const SPEC_FILE_PREFIX: &str = "Native";

pub fn lib_base_name(name: &SanitizedString) -> String {
    flat_case(name.0.as_ref()).to_string()
}

/// Returns the destination name of the built library
///
/// Example: `libsomelibrary-prebuilt.a`
pub fn dest_lib_name(name: &SanitizedString) -> String {
    format!("lib{}-prebuilt.a", flat_case(name.0.as_ref()))
}

/// Example: `some_module_impl`
pub fn impl_mod_name(name: &str) -> String {
    format!("{}_impl", snake_case(name))
}

pub fn craby_tmp_dir(project_root: &Path) -> PathBuf {
    project_root.join(".craby")
}

pub fn crate_target_dir(project_root: &Path, target: &str) -> PathBuf {
    project_root.join("target").join(target).join("release")
}

pub fn crate_dir(project_root: &Path) -> PathBuf {
    project_root.join("crates").join("lib")
}

pub fn crate_manifest_path(project_root: &Path) -> PathBuf {
    crate_dir(project_root).join("Cargo.toml")
}

pub fn cxx_bridge_dir(project_root: &Path, target: &str) -> PathBuf {
    project_root.join("target").join(target).join("cxxbridge")
}

pub fn cxx_bridge_include_dir(project_root: &Path) -> PathBuf {
    crate_dir(project_root).join("include")
}

pub fn cxx_dir(project_root: &Path) -> PathBuf {
    project_root.join("cpp")
}

pub fn android_path(project_root: &Path) -> PathBuf {
    project_root.join("android")
}

pub fn jni_base_path(project_root: &Path) -> PathBuf {
    android_path(project_root)
        .join("src")
        .join("main")
        .join("jni")
}

pub fn ios_base_path(project_root: &Path) -> PathBuf {
    project_root.join("ios")
}
