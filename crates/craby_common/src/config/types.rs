use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CargoManifest {
    pub package: PackageConfig,
    pub lib: LibConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PackageConfig {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LibConfig {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub project: ProjectConfig,
    pub android: AndroidConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectConfig {
    pub name: String,
    pub source_dir: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AndroidConfig {
    pub package_name: String,
}

#[derive(Debug)]
pub struct CompleteConfig {
    pub project: ProjectConfig,
    pub android: AndroidConfig,
    pub project_root: PathBuf,
    pub source_dir: PathBuf,
}
