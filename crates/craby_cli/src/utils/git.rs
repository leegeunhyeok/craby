use std::{fs, path::PathBuf, process::Command};

use log::debug;

use crate::utils::terminal::run_command;

pub fn is_git_available() -> bool {
    Command::new("git").arg("--version").output().is_ok()
}

pub fn clone_template() -> Result<PathBuf, anyhow::Error> {
    let temp_dir = std::env::temp_dir().join("craby-init");
    debug!("Cloning template to: {:?}", temp_dir);

    if temp_dir.try_exists()? {
        fs::remove_dir_all(&temp_dir)?;
    }
    fs::create_dir_all(&temp_dir)?;

    debug!("Cloning template...");
    run_command(
        "git",
        &[
            "clone",
            "--depth",
            "1",
            "--filter=blob:none",
            "--sparse",
            "https://github.com/leegeunhyeok/craby.git",
            temp_dir.to_str().unwrap(),
        ],
        None,
    )?;

    debug!("Setting sparse checkout...");
    run_command(
        "git",
        &["sparse-checkout", "set", "template"],
        Some(temp_dir.to_str().unwrap()),
    )?;

    let temp_dir = temp_dir.join("template");

    if !temp_dir.try_exists()? {
        anyhow::bail!("Template directory does not exist: {:?}", temp_dir);
    }

    Ok(temp_dir)
}
