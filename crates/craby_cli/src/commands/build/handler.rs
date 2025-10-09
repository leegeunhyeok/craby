use std::path::PathBuf;

use craby_build::{
    constants::toolchain::BUILD_TARGETS,
    platform::{android as android_build, ios as ios_build},
};
use craby_common::{config::load_config, env::is_initialized};
use log::info;
use owo_colors::OwoColorize;

use crate::{commands::build::guide, utils::terminal::with_spinner};

pub struct BuildOptions {
    pub project_root: PathBuf,
}

pub fn perform(opts: BuildOptions) -> anyhow::Result<()> {
    let config = load_config(&opts.project_root)?;

    if !is_initialized(&opts.project_root) {
        anyhow::bail!("Craby project is not initialized. Please run `craby init` first.");
    }

    info!("Starting to build the Cargo project...");
    with_spinner("Building Cargo projects...", |pb| {
        for (i, target) in BUILD_TARGETS.iter().enumerate() {
            pb.set_message(format!(
                "[{}/{}] Building for target: {}",
                i + 1,
                BUILD_TARGETS.len(),
                target.to_str().dimmed()
            ));
            craby_build::cargo::build::build_target(&opts.project_root, target)?;
        }
        Ok(())
    })?;
    info!("Cargo project build completed successfully");

    info!("Creating Android artifacts...");
    android_build::crate_libs(&config)?;

    info!("Creating iOS XCFramework...");
    ios_build::crate_libs(&config)?;

    info!("Build completed successfully 🎉");
    guide::print_guide(&config.project.name);

    Ok(())
}
