use std::path::PathBuf;

use craby_common::utils::path::tmp_dir;

pub fn crate_dir(project_root: &PathBuf, crate_name: &str) -> PathBuf {
    project_root.join("crates").join(crate_name)
}

pub fn crate_manifest_path(project_root: &PathBuf, crate_name: &str) -> PathBuf {
    crate_dir(project_root, crate_name).join("Cargo.toml")
}

pub fn binding_header_dir(project_root: &PathBuf) -> PathBuf {
    tmp_dir(project_root).join("include")
}

pub mod android {
    use crate::constants::android::{ABI_ARM64_V8A, ABI_ARMEABI_V7A, ABI_X86, ABI_X86_64};

    pub fn get_abi_by_target(target: &str) -> &str {
        match target {
            "aarch64-linux-android" => ABI_ARM64_V8A,
            "armv7-linux-androideabi" => ABI_ARMEABI_V7A,
            "x86_64-linux-android" => ABI_X86_64,
            "i686-linux-android" => ABI_X86,
            _ => unreachable!("Unsupported target: {}", target),
        }
    }
}
