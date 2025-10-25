use std::path::Path;

use crate::utils::git::is_git_available;

pub fn validate_env(dest_dir: &Path) -> anyhow::Result<()> {
    if dest_dir.try_exists()? {
        anyhow::bail!("{} directory already exists", dest_dir.display());
    }

    if !is_git_available() {
        anyhow::bail!("Git command is not available. Please install Git and try again.");
    }

    Ok(())
}
