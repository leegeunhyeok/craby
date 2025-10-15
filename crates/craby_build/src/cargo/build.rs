use std::{path::Path, process::Command};

use craby_common::constants::crate_manifest_path;
use log::{debug, error};

use crate::constants::toolchain::Target;

pub fn build_target(project_root: &Path, target: &Target) -> Result<(), anyhow::Error> {
    let manifest_path = crate_manifest_path(project_root)
        .to_string_lossy()
        .to_string();
    debug!("Manifest path: {}", manifest_path);

    let target_label = format!("({})", target.to_str());
    debug!("Building for target {}", target_label);

    let args = [
        "build",
        "--manifest-path",
        manifest_path.as_str(),
        "--target",
        target.to_str(),
        "--release",
    ];

    let res = match &target {
        Target::Android(abi) => Command::new("cargo")
            .args(args)
            .envs(abi.to_env()?)
            .output(),
        Target::Ios(_) => Command::new("cargo").args(args).output(),
    }?;

    if !res.status.success() {
        error!("{}", String::from_utf8_lossy(&res.stderr));
        anyhow::bail!("Failed to build (Target: {})", target.to_str());
    }

    Ok(())
}
