use std::{path::Path, process::Command};

use log::{debug, error, info};
use owo_colors::OwoColorize;

use crate::{
    constants,
    utils::path::{android_jni_libs_dir, crate_manifest_path},
};

pub fn build_targets(project_root: &Path) -> Result<(), anyhow::Error> {
    build_ios(project_root)?;
    build_android(project_root)?;

    Ok(())
}

fn build_ios(project_root: &Path) -> Result<(), anyhow::Error> {
    let manifest_path = crate_manifest_path(&project_root.to_path_buf(), "ios")
        .to_string_lossy()
        .to_string();
    debug!("Manifest path: {}", manifest_path);

    for target in constants::toolchain::TARGETS {
        if target.contains("android") {
            continue;
        }

        let target_label = format!("({})", target);
        info!("Building for iOS {}", target_label.dimmed());

        let res = Command::new("cargo")
            .args([
                "build",
                "--manifest-path",
                manifest_path.as_str(),
                "--target",
                target,
                "--release",
            ])
            .output()?;

        if !res.status.success() {
            error!("{}", String::from_utf8_lossy(&res.stderr));
            anyhow::bail!("Failed to build iOS");
        }
    }

    Ok(())
}

fn build_android(project_root: &Path) -> Result<(), anyhow::Error> {
    let manifest_path = crate_manifest_path(&project_root.to_path_buf(), "android");
    let output_dir = android_jni_libs_dir(&project_root.to_path_buf());

    let mut cmd = Command::new("cargo");
    let args = [
        "ndk",
        "--manifest-path",
        manifest_path.to_str().unwrap(),
        "-o",
        output_dir.to_str().unwrap(),
    ];
    let cmd = cmd.args(args);
    debug!("cargo ndk args: {:?}", args);

    constants::android::ABI_TARGETS.iter().for_each(|target| {
        debug!("cargo ndk target: {}", target);
        cmd.args(["-t", target]);
    });

    info!("Building for Android with NDK...");
    let res = cmd.args(["build", "--release"]).output()?;

    if !res.status.success() {
        error!("{}", String::from_utf8_lossy(&res.stderr));
        anyhow::bail!("Failed to build Android");
    }

    Ok(())
}
