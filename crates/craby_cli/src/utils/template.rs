use handlebars::Handlebars;
use log::debug;
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

pub fn render_template(template_dir: &Path, data: &BTreeMap<&str, &str>) -> anyhow::Result<()> {
    let reg = Handlebars::new();

    debug!("Rendering template {:?}", template_dir,);
    debug!("Template data: {:?}", data);

    for entry in WalkDir::new(template_dir) {
        let entry = entry?;
        let path = entry.path();

        let target_path = path.strip_prefix(template_dir)?.to_path_buf();
        let replaced_path = replace_path(&target_path, data);
        let target_path = if target_path != replaced_path {
            fs::rename(&target_path, &replaced_path)?;
            replaced_path
        } else {
            target_path
        };

        if target_path.is_file() {
            let content = fs::read_to_string(path)?;
            let rendered: String = reg.render_template(&content, data)?;

            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let mut file = File::create(&target_path)?;
            file.write_all(rendered.as_bytes())?;
        }
    }

    Ok(())
}

fn replace_path(path: &Path, data: &BTreeMap<&str, &str>) -> PathBuf {
    let mut result = path.to_string_lossy().to_string();

    for (key, value) in data {
        // Replace '{{key}}' with value
        result = result.replace(format!("{{{{{key}}}}}", key = key).as_str(), value);
    }

    PathBuf::from(result)
}
