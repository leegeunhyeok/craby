use std::{fs, path::PathBuf};

use craby_common::{
    config::load_config,
    constants::{android_path, craby_tmp_dir, ios_base_path, jni_base_path},
};
use log::{debug, info};

pub struct CleanOptions {
    pub project_root: PathBuf,
}

pub fn perform(opts: CleanOptions) -> anyhow::Result<()> {
    if let Err(e) = load_config(&opts.project_root) {
        anyhow::bail!("Craby project is not initialized. reason: {}", e)
    };

    info!("🧹 Cleaning up files...");

    let cargo_target_dir = opts.project_root.join("target");
    let android_build_dir = android_path(&opts.project_root).join("build");
    let android_cxx_dir = android_path(&opts.project_root).join(".cxx");
    let android_libs_dir = jni_base_path(&opts.project_root).join("libs");
    let ios_framework_dir = ios_base_path(&opts.project_root).join("framework");
    let tmp_dir = craby_tmp_dir(&opts.project_root);

    for dir in [
        cargo_target_dir,
        android_build_dir,
        android_cxx_dir,
        android_libs_dir,
        ios_framework_dir,
        tmp_dir,
    ] {
        if dir.try_exists()? {
            debug!("Removing directory: {}", dir.display());
            fs::remove_dir_all(dir)?;
        }
    }

    info!("Done!");

    Ok(())
}
