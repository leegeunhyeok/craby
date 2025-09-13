use std::{fs, path::PathBuf};

pub fn collect_files(dir: &PathBuf, exts: &[&str]) -> Result<Vec<PathBuf>, anyhow::Error> {
    let mut files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let ext = path.extension().unwrap_or_default();
            let is_target = exts.contains(&ext.to_str().unwrap_or_default());

            if is_target {
                files.push(path);
            }
        } else if path.is_dir() {
            files.extend(collect_files(&path, exts)?);
        }
    }

    Ok(files)
}
