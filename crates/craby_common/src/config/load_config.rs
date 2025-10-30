use std::{
    fs,
    path::{Path, PathBuf},
};

use log::debug;

use crate::{
    constants::crate_dir,
    utils::{android::is_valid_android_package_name, cargo::cargo_version, string::flat_case},
};

use super::{types::Config, CargoManifest, CompleteConfig};

pub fn load_config(project_root: &Path) -> Result<CompleteConfig, anyhow::Error> {
    debug!("Cargo version: {}", cargo_version()?);
    let manifest_path = crate_dir(project_root).join("Cargo.toml");
    let config_path = project_root.join("craby.toml");

    validate_manifest(&manifest_path, &config_path)?;

    let config = fs::read_to_string(config_path)?;
    let config = toml::from_str::<Config>(&config)?;
    let source_dir = project_root.join(PathBuf::from(&config.project.source_dir));

    validate_config(&config)?;

    Ok(CompleteConfig {
        project_root: project_root.to_path_buf(),
        project: config.project,
        android: config.android,
        source_dir,
    })
}

fn validate_manifest(
    manifest_path: &PathBuf,
    config_path: &PathBuf,
) -> Result<Config, anyhow::Error> {
    if !manifest_path.try_exists()? {
        return Err(anyhow::anyhow!("Cargo.toml not found"));
    }

    if !config_path.try_exists()? {
        return Err(anyhow::anyhow!("craby.toml not found"));
    }

    let manifest = fs::read_to_string(manifest_path)?;
    let manifest = toml::from_str::<CargoManifest>(&manifest)?;

    let config = fs::read_to_string(config_path)?;
    let config = toml::from_str::<Config>(&config)?;

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

fn validate_config(config: &Config) -> Result<(), anyhow::Error> {
    if !is_valid_android_package_name(&config.android.package_name)? {
        anyhow::bail!(format!(
            "Invalid Android package name: {}",
            config.android.package_name
        ));
    }

    Ok(())
}
