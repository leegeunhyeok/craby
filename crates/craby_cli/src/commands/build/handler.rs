use std::path::PathBuf;

use craby_build::{
    constants::{android::Abi, ios::Identifier, toolchain::Target},
    platform::{android as android_build, ios as ios_build},
};
use craby_common::{config::load_config, env::is_initialized};
use log::info;
use owo_colors::OwoColorize;

use crate::{commands::build::guide, utils::terminal::with_spinner};

const BUILD_TARGETS: [Target; 6] = [
    Target::Android(Abi::Arm64V8a),
    Target::Android(Abi::ArmeAbiV7a),
    Target::Android(Abi::X86_64),
    Target::Android(Abi::X86),
    Target::Ios(Identifier::Arm64),
    Target::Ios(Identifier::Arm64Simulator),
];

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
        BUILD_TARGETS
            .iter()
            .enumerate()
            .try_for_each(|(i, target)| -> anyhow::Result<()> {
                pb.set_message(format!(
                    "[{}/{}] Building for target: {}",
                    i + 1,
                    BUILD_TARGETS.len(),
                    target.to_str().dimmed()
                ));
                craby_build::cargo::build::build_target(&opts.project_root, target)?;
                Ok(())
            })?;
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
