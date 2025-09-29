use std::{fs, path::PathBuf, process::Command};

pub fn is_git_available() -> bool {
    Command::new("git").arg("--version").output().is_ok()
}

pub fn clone_template() -> Result<PathBuf, anyhow::Error> {
    let temp_dir = std::env::temp_dir().join("craby-init");

    if fs::exists(&temp_dir)? {
        fs::remove_dir_all(&temp_dir)?;
    }

    Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "--filter=blob:none",
            "-b",
            "template",
            "--quiet",
            "--sparse",
            "https://github.com/leegeunhyeok/craby.git",
            temp_dir.to_str().unwrap(),
        ])
        .output()?;

    Command::new("git")
        .args(["sparse-checkout", "set", "template/"])
        .current_dir(&temp_dir)
        .status()?;

    Ok(temp_dir.join("template"))
}
