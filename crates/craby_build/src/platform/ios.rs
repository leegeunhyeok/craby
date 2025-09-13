use std::{fs, path::PathBuf};

use crate::{
    cargo::artifact::{ArtifactType, Artifacts},
    constants::{ios::Identifier, toolchain::Target},
};
use craby_common::config::CompleteCrabyConfig;
use log::debug;

pub fn crate_libs<'a>(config: &'a CompleteCrabyConfig) -> Result<(), anyhow::Error> {
    let ios_base_path = ios_base_path(&config.project_root);

    if ios_base_path.exists() {
        fs::remove_dir_all(&ios_base_path)?;
        debug!("Cleaned up existing iOS base directory");
    }

    for target in [
        Target::Ios(Identifier::Arm64),
        Target::Ios(Identifier::Arm64Simulator),
    ] {
        if let Target::Ios(identifier) = &target {
            let artifacts = Artifacts::get_artifacts(config, &target)?;
            let identifier = identifier.to_str();

            // {ios_base_path}/src
            artifacts.copy_to(ArtifactType::Src, &ios_base_path.join("src"))?;

            // {ios_base_path}/include
            artifacts.copy_to(ArtifactType::Header, &ios_base_path.join("include"))?;

            // {ios_base_path}/libs/{lib_identifier}
            artifacts.copy_to(
                ArtifactType::Lib,
                &ios_base_path.join("libs").join(identifier),
            )?;
        } else {
            unreachable!();
        }
    }

    Ok(())
}

fn ios_base_path(project_root: &PathBuf) -> PathBuf {
    project_root.join("ios")
}
