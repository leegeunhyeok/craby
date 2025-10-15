use std::path::PathBuf;

use craby_common::{config::CompleteCrabyConfig, constants::jni_base_path};
use log::debug;

use crate::{
    cargo::artifact::{ArtifactType, Artifacts},
    constants::{android::Abi, toolchain::Target},
    cxx::{replace_cxx_header, replace_cxx_iter_template},
};

pub const ANDROID_TARGETS: [Target; 4] = [
    Target::Android(Abi::Arm64V8a),
    Target::Android(Abi::ArmeAbiV7a),
    Target::Android(Abi::X86_64),
    Target::Android(Abi::X86),
];

pub fn crate_libs(config: &CompleteCrabyConfig) -> Result<(), anyhow::Error> {
    let jni_base_path = jni_base_path(&config.project_root);

    for target in ANDROID_TARGETS {
        debug!("Copying artifacts to JNI base path: {:?}", jni_base_path);

        if let Target::Android(abi) = &target {
            let artifacts = Artifacts::get_artifacts(config, &target)?;
            let abi = abi.to_str();

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

    let signal_path = jni_base_path.join("include").join("signals.h");
    debug!("Post-processing signals.h: {:?}", signal_path);
    if signal_path.try_exists()? {
        replace_cxx_header(&signal_path)?;
    }

    let cxx_path = jni_base_path.join("include").join("cxx.h");
    debug!("Post-processing cxx.h: {:?}", cxx_path);
    if cxx_path.try_exists()? {
        replace_cxx_iter_template(&cxx_path)?;
    }

    Ok(())
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
