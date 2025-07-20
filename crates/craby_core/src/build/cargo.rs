use std::{path::Path, process::Command};

use craby_common::constants;
use log::{debug, error, info};

pub fn build_targets(project_root: &Path) -> Result<(), anyhow::Error> {
    build_ios(project_root)?;
    build_android(project_root)?;

    Ok(())
}

fn build_ios(project_root: &Path) -> Result<(), anyhow::Error> {
    let manifest_path = project_root
        .join("crates/ios/Cargo.toml")
        .to_string_lossy()
        .to_string();
    debug!("Manifest path: {}", manifest_path);

    for target in constants::toolchain::TARGETS {
        if target.contains("android") {
            continue;
        }

        info!("Building for iOS (target: {})", target);

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
    let manifest_path = project_root
        .join("crates/android/Cargo.toml")
        .to_string_lossy()
        .to_string();
    let output_dir = project_root
        .join("android/src/main/jniLibs")
        .to_string_lossy()
        .to_string();

    let mut cmd = Command::new("cargo");
    let cmd = cmd.args([
        "ndk",
        "--manifest-path",
        manifest_path.as_str(),
        "-o",
        output_dir.as_str(),
    ]);

    constants::android::ABI_TARGETS.iter().for_each(|target| {
        cmd.arg("-t").arg(target);
    });

    info!("Building Android with NDK...");
    let res = cmd.args(["build", "--release"]).output()?;

    if !res.status.success() {
        error!("{}", String::from_utf8_lossy(&res.stderr));
        anyhow::bail!("Failed to build Android");
    }

    Ok(())
}
