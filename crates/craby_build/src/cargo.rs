use std::{path::Path, process::Command};

use log::{debug, error, info};
use owo_colors::OwoColorize;

use crate::{constants, utils::crate_manifest_path};

pub fn build_targets(project_root: &Path) -> Result<(), anyhow::Error> {
    let manifest_path = crate_manifest_path(&project_root.to_path_buf(), "lib")
        .to_string_lossy()
        .to_string();
    debug!("Manifest path: {}", manifest_path);

    for target in constants::toolchain::TARGETS {
        let target_label = format!("({})", target);
        info!("Building for target {}", target_label.dimmed());

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
            anyhow::bail!("Failed to build (Target: {})", target);
        }
    }

    Ok(())
}
