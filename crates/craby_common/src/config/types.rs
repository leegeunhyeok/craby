use std::path::PathBuf;

use regex::Regex;
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
    pub codegen: CodegenConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectConfig {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CodegenConfig {
    pub exclude: Vec<String>,
    pub include: Vec<String>,
}

#[derive(Debug)]
pub struct CompleteCrabyConfig {
    pub project_root: PathBuf,
    pub project: ProjectConfig,
    pub codegen: CompleteCodegenConfig,
}

#[derive(Debug)]
pub struct CompleteCodegenConfig {
    pub exclude: Vec<Regex>,
    pub include: Vec<Regex>,
}

impl CompleteCrabyConfig {
    pub fn is_included_method(&self, name: &String) -> bool {
        self.codegen.include.iter().any(|re| re.is_match(name))
    }

    pub fn is_excluded_method(&self, name: &String) -> bool {
        self.codegen.exclude.iter().any(|re| re.is_match(name))
    }
}
