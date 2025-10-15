use handlebars::Handlebars;
use log::debug;
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

pub fn render_template(
    dest_dir: &Path,
    template_dir: &Path,
    template_data: &BTreeMap<&str, &str>,
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
            let rendered = custom_render(&target_path, &rendered).unwrap_or(rendered);

            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let mut file = File::create(&target_path)?;
            file.write_all(rendered.as_bytes())?;
        }
    }

    fs::rename(template_dir, dest_dir)?;

    Ok(())
}

fn replace_path(
    path: &Path,
    template_data: &BTreeMap<&str, &str>,
    keep_base_name: bool,
) -> PathBuf {
    if keep_base_name {
        let base_name = path.file_name().unwrap().to_string_lossy().to_string();
        let mut parent = path.parent().unwrap().to_string_lossy().to_string();

        for (key, value) in template_data {
            // Replace '{{key}}' with given value
            parent = parent.replace(format!("{{{{{key}}}}}", key = key).as_str(), value);
        }

        PathBuf::from(parent).join(base_name)
    } else {
        let mut result = path.to_string_lossy().to_string();

        for (key, value) in template_data {
            // Replace '{{key}}' with given value
            result = result.replace(format!("{{{{{key}}}}}", key = key).as_str(), value);
        }

        PathBuf::from(result)
    }
}

/// Custom render rules for specific files
fn custom_render(path: &Path, content: &str) -> Option<String> {
    let base_name = path.file_name().unwrap().to_string_lossy().to_string();

    match base_name.as_str() {
        "biome.json" => Some(content.replace("\"root\": false", "\"root\": true")),
        _ => None,
    }
}
