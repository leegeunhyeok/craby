use std::{fs, path::PathBuf, process};

use log::error;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CrabyConfig {
    codegen: CodegenConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CodegenConfig {
    exclude: Vec<String>,
    include: Vec<String>,
}

#[derive(Debug)]
pub struct CompleteCrabyConfig {
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

impl CrabyConfig {
    pub fn into_complete(&self) -> CompleteCrabyConfig {
        let handle_error = |e: regex::Error| {
            error!("Invalid regex: {}", e);
            process::exit(1);
        };

        let codegen = CompleteCodegenConfig {
            exclude: self
                .codegen
                .exclude
                .iter()
                .map(|s| match Regex::new(s) {
                    Ok(re) => re,
                    Err(e) => handle_error(e),
                })
                .collect(),
            include: self
                .codegen
                .include
                .iter()
                .map(|s| match Regex::new(s) {
                    Ok(re) => re,
                    Err(e) => handle_error(e),
                })
                .collect(),
        };

        CompleteCrabyConfig { codegen }
    }
}

pub fn load_config(project_root: &PathBuf) -> Result<CrabyConfig, anyhow::Error> {
    let config_path = project_root.join("craby.toml");

    if !config_path.exists() {
        return Err(anyhow::anyhow!("craby.toml not found"));
    }

    let config = fs::read_to_string(config_path)?;
    let config = toml::from_str::<CrabyConfig>(&config)?;
    Ok(config)
}
