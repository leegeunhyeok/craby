use std::{path::PathBuf, process::Command};

use craby_common::{config::CompleteConfig, constants::jni_base_path};
use log::{debug, info};
use owo_colors::OwoColorize;

use crate::{
    cargo::artifact::{ArtifactType, Artifacts},
    constants::toolchain::Target,
    platform::{
        android::path::ndk_llvm_strip_path,
        common::{replace_cxx_header, replace_cxx_iter_template},
    },
};

pub fn crate_libs(config: &CompleteConfig, build_targets: &[Target]) -> Result<(), anyhow::Error> {
    let jni_base_path = jni_base_path(&config.project_root);

    for target in build_targets {
        debug!("Copying artifacts to JNI base path: {:?}", jni_base_path);

        if let Target::Android(abi) = target {
            let artifacts = Artifacts::get_artifacts(config, target)?;
            let abi = abi.to_str();

            artifacts.path_of(ArtifactType::Lib).iter().try_for_each(
                |lib| -> Result<(), anyhow::Error> {
                    info!(
                        "Optimizing library... {}",
                        format!("({})", artifacts.identifier).dimmed()
                    );
                    strip_lib(lib)?;
                    Ok(())
                },
            )?;

            // android/src/main/jni/src
            artifacts.copy_to(ArtifactType::Src, &jni_base_path.join("src"))?;

            // android/src/main/jni/include
            artifacts.copy_to(ArtifactType::Header, &jni_base_path.join("include"))?;

            // android/src/main/jni/libs/{abi}
            artifacts.copy_to(ArtifactType::Lib, &jni_base_path.join("libs").join(abi))?;
        }
    }

    let signal_path = jni_base_path.join("include").join("CrabySignals.h");
    debug!("Post-processing CrabySignals.h: {:?}", signal_path);
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

fn strip_lib(lib: &PathBuf) -> Result<(), anyhow::Error> {
    let bin = ndk_llvm_strip_path()?;
    let res = Command::new(bin)
        .arg("--strip-unneeded")
        .arg(lib)
        .output()?;

    if !res.status.success() {
        anyhow::bail!(
            "Failed to strip library: {}",
            String::from_utf8_lossy(&res.stderr)
        );
    }

    Ok(())
}

pub mod path {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use crate::constants::android::Abi;

    pub fn ndk_bin_path() -> Result<PathBuf, anyhow::Error> {
        let os_path = match std::env::consts::OS {
            "macos" => Ok("darwin-x86_64"),
            "linux" => Ok("linux-x86_64"),
            "windows" => Ok("windows-x86_64"),
            _ => Err(anyhow::anyhow!("Unsupported OS: {}", std::env::consts::OS)),
        }?;

        let path = ndk_root_path()?
            .join("toolchains")
            .join("llvm")
            .join("prebuilt")
            .join(os_path)
            .join("bin");

        Ok(path)
    }

    pub fn ndk_clang_path(abi: &Abi, cxx: bool) -> Result<PathBuf, anyhow::Error> {
        let ndk_bin_path = ndk_bin_path()?;
        let clang_name = abi.to_clang_name(cxx);

        Ok(ndk_bin_path.join(clang_name))
    }

    pub fn ndk_llvm_ar_path() -> Result<PathBuf, anyhow::Error> {
        Ok(ndk_bin_path()?.join("llvm-ar"))
    }

    pub fn ndk_llvm_strip_path() -> Result<PathBuf, anyhow::Error> {
        Ok(ndk_bin_path()?.join("llvm-strip"))
    }

    fn ndk_root_path() -> Result<PathBuf, anyhow::Error> {
        if let Ok(path) = std::env::var("ANDROID_NDK_HOME") {
            return Ok(PathBuf::from(path));
        }

        let ndk_dir = PathBuf::from(std::env::var("ANDROID_HOME").map_err(|_| {
            anyhow::anyhow!("`ANDROID_NDK_HOME` or `ANDROID_HOME` environment variable is not set")
        })?)
        .join("ndk");

        latest_ndk_path(&ndk_dir)
            .ok_or_else(|| anyhow::anyhow!("Android NDK not found in {}", ndk_dir.display()))
    }

    fn latest_ndk_path(ndk_dir: &Path) -> Option<PathBuf> {
        fn ndk_version_key(path: &Path) -> Option<Vec<u64>> {
            path.file_name()?
                .to_string_lossy()
                .split('.')
                .map(|part| part.parse::<u64>())
                .collect::<Result<Vec<_>, _>>()
                .ok()
        }

        fs::read_dir(ndk_dir)
            .ok()?
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                path.is_dir().then_some(path)
            })
            .filter_map(|path| ndk_version_key(&path).map(|key| (key, path)))
            .max_by(|(left, _), (right, _)| left.cmp(right))
            .map(|(_, path)| path)
    }
}
