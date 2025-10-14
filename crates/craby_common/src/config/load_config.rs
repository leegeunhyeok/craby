use std::{fs, path::PathBuf};

use log::debug;

use crate::{
    constants::crate_dir,
    utils::{cargo::cargo_version, string::flat_case},
};

use super::{types::CrabyConfig, CargoManifest, CompleteCrabyConfig};

pub fn load_config(project_root: &PathBuf) -> Result<CompleteCrabyConfig, anyhow::Error> {
    debug!("Cargo version: {}", cargo_version()?);
    let manifest_path = crate_dir(project_root).join("Cargo.toml");
    let config_path = project_root.join("craby.toml");

    validate_config(&manifest_path, &config_path)?;

    let config = fs::read_to_string(config_path)?;
    let config = toml::from_str::<CrabyConfig>(&config)?;
    let source_dir = project_root.join(PathBuf::from(&config.project.source_dir));

    Ok(CompleteCrabyConfig {
        project_root: project_root.clone(),
        project: config.project,
        source_dir,
    })
}

fn validate_config(
    manifest_path: &PathBuf,
    config_path: &PathBuf,
) -> Result<CrabyConfig, anyhow::Error> {
    if !manifest_path.try_exists()? {
        return Err(anyhow::anyhow!("Cargo.toml not found"));
    }

    if !config_path.try_exists()? {
        return Err(anyhow::anyhow!("craby.toml not found"));
    }

    let manifest = fs::read_to_string(manifest_path)?;
    let manifest = toml::from_str::<CargoManifest>(&manifest)?;

    let config = fs::read_to_string(config_path)?;
    let config = toml::from_str::<CrabyConfig>(&config)?;

    if manifest.package.name != config.project.name {
        return Err(anyhow::anyhow!(format!(
            "Craby project name({}) does not match Cargo project name({})",
            config.project.name, manifest.lib.name,
        )));
    }

    let expected_lib_name = flat_case(&config.project.name);
    if manifest.lib.name != expected_lib_name {
        return Err(anyhow::anyhow!(format!(
            "Invalid library name in Cargo.toml: {} (Expected: {})",
            manifest.lib.name, expected_lib_name,
        )));
    }

    if config.project.source_dir.is_empty() {
        return Err(anyhow::anyhow!("Source directory is not set"));
    }

    Ok(config)
}
