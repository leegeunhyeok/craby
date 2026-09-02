use handlebars::Handlebars;
use log::debug;
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

pub type TemplateData = BTreeMap<&'static str, String>;

pub fn render_template(
    dest_dir: &Path,
    template_dir: &Path,
    template_data: &BTreeMap<&str, String>,
) -> anyhow::Result<()> {
    let reg = Handlebars::new();

    debug!(
        "Rendering template {:?} with data {:#?}",
        template_dir, template_data
    );

    for entry in WalkDir::new(template_dir) {
        let entry = entry?;
        let path = entry.path();
        let base_bath = replace_path(path, template_data, true);
        let target_path = replace_path(path, template_data, false);

        if base_bath != target_path {
            debug!("Renaming {:?} to {:?}", base_bath, target_path);
            fs::rename(&base_bath, &target_path)?;
        }

        if target_path.is_dir() {
            fs::create_dir_all(&target_path)?;
        } else if target_path.is_file() {
            debug!("Processing {:?}", target_path);
            let content = fs::read_to_string(&target_path)?;
            let rendered = reg.render_template(&content, template_data)?;

            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let mut file = File::create(&target_path)?;
            file.write_all(rendered.as_bytes())?;
        }
    }

    move_dir(template_dir, dest_dir)?;

    Ok(())
}

fn move_dir(source: &Path, dest: &Path) -> io::Result<()> {
    match fs::rename(source, dest) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::CrossesDevices => {
            copy_dir_all(source, dest)?;
            fs::remove_dir_all(source)
        }
        Err(e) => Err(e),
    }
}

fn copy_dir_all(source: &Path, dest: &Path) -> io::Result<()> {
    fs::create_dir_all(dest)?;

    for entry in WalkDir::new(source).min_depth(1) {
        let entry = entry?;
        let path = entry.path();
        let target_path = dest.join(
            path.strip_prefix(source)
                .expect("path must be under source"),
        );

        if path.is_dir() {
            fs::create_dir_all(&target_path)?;
        } else if path.is_file() {
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(path, target_path)?;
        }
    }

    Ok(())
}

fn replace_path(
    path: &Path,
    template_data: &BTreeMap<&str, String>,
    keep_base_name: bool,
) -> PathBuf {
    if keep_base_name {
        let base_name = path.file_name().unwrap().to_string_lossy().to_string();
        let mut parent = path.parent().unwrap().to_string_lossy().to_string();

        for (key, value) in template_data {
            // Replace '{{key}}' with given value
            parent = parent.replace(&format!("{{{{{key}}}}}", key = key), value);
        }

        PathBuf::from(parent).join(base_name)
    } else {
        let mut result = path.to_string_lossy().to_string();

        for (key, value) in template_data {
            // Replace '{{key}}' with given value
            result = result.replace(&format!("{{{{{key}}}}}", key = key), value);
        }

        PathBuf::from(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn copy_dir_all_copies_nested_files() {
        let test_dir = std::env::temp_dir().join(format!(
            "craby-template-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let source = test_dir.join("source");
        let dest = test_dir.join("dest");

        fs::create_dir_all(source.join("nested")).unwrap();
        fs::write(source.join("root.txt"), "root").unwrap();
        fs::write(source.join("nested").join("child.txt"), "child").unwrap();

        copy_dir_all(&source, &dest).unwrap();

        assert_eq!(fs::read_to_string(dest.join("root.txt")).unwrap(), "root");
        assert_eq!(
            fs::read_to_string(dest.join("nested").join("child.txt")).unwrap(),
            "child"
        );

        fs::remove_dir_all(test_dir).unwrap();
    }
}
