use std::path::PathBuf;

use craby_common::utils::fs::clean_binding_headers;
use log::info;

pub struct CleanOptions {
    pub project_root: PathBuf,
}

pub fn r#impl(opts: CleanOptions) -> anyhow::Result<()> {
    info!("🧹 Cleaning up temporary files...");

    clean_binding_headers(&opts.project_root)?;

    info!("Done!");

    Ok(())
}
