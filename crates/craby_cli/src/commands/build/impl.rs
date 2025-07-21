use std::path::PathBuf;

use craby_common::{
    build::{self, xcode::CreateXcframeworkOptions},
    env::is_initialized,
    utils::sanitize_str,
};
use log::info;

use crate::commands::build::guide;

pub struct BuildOptions {
    pub project_root: PathBuf,
    pub lib_name: String,
}

pub fn r#impl(opts: BuildOptions) -> anyhow::Result<()> {
    let lib_name = sanitize_str(&opts.lib_name).replace("_", "");

    if !is_initialized(&opts.project_root) {
        anyhow::bail!("Craby project is not initialized. Please run `craby init` first.");
    }

    info!("Building Cargo projects...");
    build::cargo::build_targets(&opts.project_root)?;

    info!("Generating C bindings...");
    let output = build::c::generate_c_bindings(&opts.project_root, &lib_name)?;

    info!("Creating xcframework...");
    build::xcode::create_xcframework(CreateXcframeworkOptions {
        project_root: opts.project_root,
        header_path: output,
        lib_name: lib_name.clone(),
    })?;

    info!("Build completed successfully 🎉");
    guide::print_guide(&lib_name);

    Ok(())
}
