use std::{fs, path::PathBuf};

use craby_common::utils::path::tmp_dir;
use log::info;

pub struct CleanOptions {
    pub project_root: PathBuf,
}

pub fn r#impl(opts: CleanOptions) -> anyhow::Result<()> {
    info!("🧹 Cleaning up temporary files...");

    let target = tmp_dir(&opts.project_root);
    let files = fs::read_dir(target)?;
    for file in files {
        let file = file?;
        fs::remove_file(file.path())?;
    }

    info!("Done!");

    Ok(())
}
