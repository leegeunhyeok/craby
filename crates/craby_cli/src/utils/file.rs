use std::{fs, path::PathBuf};

pub fn write_file(path: PathBuf, content: String, overwrite: bool) -> anyhow::Result<bool> {
    if overwrite == false && fs::exists(&path)? {
        return Ok(false);
    }

    fs::write(path, content)?;
    Ok(true)
}
