use std::{fs, path::PathBuf};

pub fn write_file(file_path: &PathBuf, content: &String, overwrite: bool) -> anyhow::Result<bool> {
    if !overwrite && file_path.try_exists()? {
        return Ok(false);
    }

    if let Some(parent) = file_path.parent() {
        if !parent.try_exists()? {
            fs::create_dir_all(parent)?;
        }
    }

    fs::write(file_path, content)?;
    Ok(true)
}
