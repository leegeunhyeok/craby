use std::{path::Path, process::Command};

use craby_common::constants::crate_manifest_path;
use log::{debug, error, info};
use owo_colors::OwoColorize;

use crate::constants::{android::Abi, ios::Identifier, toolchain::Target};

pub fn build_all_targets(project_root: &Path) -> Result<(), anyhow::Error> {
    let manifest_path = crate_manifest_path(&project_root.to_path_buf())
        .to_string_lossy()
        .to_string();
    debug!("Manifest path: {}", manifest_path);

    for target in [
        Target::Android(Abi::Arm64V8a),
        Target::Android(Abi::ArmeAbiV7a),
        Target::Android(Abi::X86_64),
        Target::Android(Abi::X86),
        Target::Ios(Identifier::Arm64),
        Target::Ios(Identifier::Arm64Simulator),
    ] {
        let target_label = format!("({})", target.to_str());
        info!("Building for target {}", target_label.dimmed());

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
    }

    Ok(())
}
