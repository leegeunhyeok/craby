use std::{fs, path::PathBuf};

use craby_common::config::CompleteCrabyConfig;
use log::debug;

use crate::{
    cargo::artifact::{ArtifactType, Artifacts},
    constants::{android::Abi, toolchain::Target},
};

const ANDROID_TARGETS: [Target; 4] = [
    Target::Android(Abi::Arm64V8a),
    Target::Android(Abi::ArmeAbiV7a),
    Target::Android(Abi::X86_64),
    Target::Android(Abi::X86),
];

pub fn crate_libs<'a>(config: &'a CompleteCrabyConfig) -> Result<(), anyhow::Error> {
    let android_path = android_path(&config.project_root);
    let jni_base_path = jni_base_path(&config.project_root);

    if jni_base_path.exists() {
        fs::remove_dir_all(&jni_base_path)?;
        debug!("Cleaned up existing JNI base directory");
    }

    for target in ANDROID_TARGETS {
        if let Target::Android(abi) = &target {
            let artifacts = Artifacts::get_artifacts(config, &target)?;
            let abi = abi.to_str();

            debug!("Copying artifacts to JNI base path: {:?}", jni_base_path);

            // android/src/main/jni/src
            artifacts.copy_to(ArtifactType::Src, &jni_base_path.join("src"))?;

            // android/src/main/jni/include
            artifacts.copy_to(ArtifactType::Header, &jni_base_path.join("include"))?;

            // android/src/main/jni/libs/{abi}
            artifacts.copy_to(ArtifactType::Lib, &jni_base_path.join("libs").join(abi))?;
        } else {
            unreachable!();
        }
    }

    // android/CMakeLists.txt
    fs::write(
        android_path.join("CMakeLists.txt"),
        craby_codegen::platform::android::template::cmakelists(&config.project.name),
    )?;

    // android/src/main/jni/OnLoad.cpp
    fs::write(
        jni_base_path.join("OnLoad.cpp"),
        craby_codegen::platform::android::template::cxx_on_load(&config.project.name),
    )?;

    Ok(())
}

fn android_path(project_root: &PathBuf) -> PathBuf {
    project_root.join("android")
}

fn jni_base_path(project_root: &PathBuf) -> PathBuf {
    android_path(project_root)
        .join("src")
        .join("main")
        .join("jni")
}

pub fn get_ndk_bin_path() -> Result<PathBuf, anyhow::Error> {
    let os_path = match std::env::consts::OS {
        "macos" => Ok("darwin-x86_64"),
        "linux" => Ok("linux-x86_64"),
        "windows" => Ok("windows-x86_64"),
        _ => Err(anyhow::anyhow!("Unsupported OS: {}", std::env::consts::OS)),
    }?;

    let path = PathBuf::from(
        std::env::var("ANDROID_NDK_HOME")
            .expect("`ANDROID_NDK_HOME` environment variable is not set"),
    )
    .join("toolchains")
    .join("llvm")
    .join("prebuilt")
    .join(os_path)
    .join("bin");

    Ok(path)
}

pub fn get_ndk_clang_path(abi: &Abi, cxx: bool) -> Result<PathBuf, anyhow::Error> {
    let ndk_bin_path: PathBuf = get_ndk_bin_path()?;
    let clang_name = abi.to_clang_name(cxx);

    Ok(ndk_bin_path.join(clang_name))
}

pub fn get_ndk_llvm_ar_path() -> Result<PathBuf, anyhow::Error> {
    Ok(get_ndk_bin_path()?.join("llvm-ar"))
}
