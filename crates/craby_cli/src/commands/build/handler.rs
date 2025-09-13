use std::path::PathBuf;

use craby_build::platform::{android as android_build, ios as ios_build};
use craby_common::{config::load_config, env::is_initialized, utils::string::SanitizedString};
use log::info;

use crate::commands::build::guide;

pub struct BuildOptions {
    pub project_root: PathBuf,
}

pub fn perform(opts: BuildOptions) -> anyhow::Result<()> {
    let config = load_config(&opts.project_root)?;
    let lib_name = SanitizedString::from(&config.project.name);

    if !is_initialized(&opts.project_root) {
        anyhow::bail!("Craby project is not initialized. Please run `craby init` first.");
    }

    info!("Building Cargo projects...");
    craby_build::cargo::build::build_all_targets(&opts.project_root)?;

    info!("Creating Android source files...");
    android_build::crate_libs(&config)?;
    // TODO: Create OnLoad.cpp

    info!("Creating iOS source files...");
    ios_build::crate_libs(&config)?;
    // TODO: Create ModuleProvider.mm

    info!("Build completed successfully 🎉");
    guide::print_guide(&lib_name);

    Ok(())
}
