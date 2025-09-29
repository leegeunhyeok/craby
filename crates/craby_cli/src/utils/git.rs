use std::path::PathBuf;

pub fn is_git_available() -> bool {
    std::process::Command::new("git")
        .arg("--version")
        .output()
        .is_ok()
}

pub fn clone_template() -> Result<PathBuf, anyhow::Error> {
    let temp_dir = std::env::temp_dir().join("craby-init");

    std::process::Command::new("git")
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

    Ok(temp_dir)
}
