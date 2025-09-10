use std::path::PathBuf;

use craby_build::platform::{android as android_build, ios as ios_build};
use craby_common::{config::load_config, env::is_initialized, utils::string::sanitize};
use log::info;

use crate::commands::build::guide;

pub struct BuildOptions {
    pub project_root: PathBuf,
}

pub fn perform(opts: BuildOptions) -> anyhow::Result<()> {
    let config = load_config(&opts.project_root)?.into_complete();
    let lib_name = sanitize(&config.name);

    if !is_initialized(&opts.project_root) {
        anyhow::bail!("Craby project is not initialized. Please run `craby init` first.");
    }

    info!("Building Cargo projects...");
    craby_build::cargo::build_targets(&opts.project_root)?;

    info!("Generating C bindings...");
    let header_path = craby_build::c::generate_c_bindings(&opts.project_root, &lib_name)?;

    info!("Creating Android ABI files...");
    android_build::create_abi_files(android_build::CreateAbiFilesOptions {
        project_root: opts.project_root.clone(),
        header_path: header_path.clone(),
        lib_name: lib_name.clone(),
    })?;

    info!("Creating xcframework...");
    ios_build::create_xcframework(ios_build::CreateXcframeworkOptions {
        project_root: opts.project_root.clone(),
        header_path: header_path.clone(),
        lib_name: lib_name.clone(),
    })?;

    info!("Build completed successfully 🎉");
    guide::print_guide(&lib_name);

    Ok(())
}
