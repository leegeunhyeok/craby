use convert_case::{Case, Casing};
use regex::Regex;

use crate::env::Platform;

pub fn sanitize_str(value: &str) -> String {
    let re = Regex::new(r"[^a-zA-Z]").unwrap();
    re.replace_all(&value, "_").to_case(Case::Snake).to_string()
}

pub fn to_lib_name(name: &String, platform: Platform) -> String {
    match platform {
        Platform::Android => format!("lib{}.so", name),
        Platform::iOS => format!("lib{}.a", name),
    }
}

pub mod path {
    use std::path::PathBuf;

    use crate::constants::TEMP_DIR;

    pub fn tmp_dir(project_root: &PathBuf) -> PathBuf {
        project_root.join(TEMP_DIR)
    }

    pub fn crate_dir(project_root: &PathBuf, crate_name: &str) -> PathBuf {
        project_root.join("crates").join(crate_name)
    }

    pub fn crate_manifest_path(project_root: &PathBuf, crate_name: &str) -> PathBuf {
        crate_dir(project_root, crate_name).join("Cargo.toml")
    }

    pub fn crate_target_dir(project_root: &PathBuf, target: &String) -> PathBuf {
        project_root.join("target").join(target).join("release")
    }

    pub fn android_jni_libs_dir(project_root: &PathBuf) -> PathBuf {
        project_root
            .join("android")
            .join("src")
            .join("main")
            .join("jniLibs")
    }

    pub fn ios_framework_path(project_root: &PathBuf, lib_name: &String) -> PathBuf {
        project_root
            .join("ios")
            .join("framework")
            .join(format!("lib{}.xcframework", lib_name))
    }

    pub fn binding_header_dir(project_root: &PathBuf) -> PathBuf {
        tmp_dir(project_root).join("include")
    }
}
