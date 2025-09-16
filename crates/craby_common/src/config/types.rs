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
pub struct CrabyConfig {
    pub project: ProjectConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectConfig {
    pub name: String,
}

#[derive(Debug)]
pub struct CompleteCrabyConfig {
    pub project_root: PathBuf,
    pub project: ProjectConfig,
}
