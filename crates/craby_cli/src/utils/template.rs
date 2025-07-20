use handlebars::Handlebars;
use log::{debug, warn};
use owo_colors::OwoColorize;
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
    path::Path,
};
use walkdir::WalkDir;

pub fn render_template(
    template_dir: &Path,
    dest_dir: &Path,
    data: &BTreeMap<&str, &str>,
) -> anyhow::Result<()> {
    let reg = Handlebars::new();

    debug!(
        "Generating template from {:?} to {:?}",
        template_dir, dest_dir
    );
    debug!("Template data: {:?}", data);

    for entry in WalkDir::new(template_dir) {
        let entry = entry?;
        let path = entry.path();

        let rel_path = path.strip_prefix(template_dir)?;
        let rendered_rel_path = reg.render_template(&rel_path.to_string_lossy(), data)?;
        let dest_path = dest_dir.join(
            rendered_rel_path
                .strip_suffix(".hbs")
                .unwrap_or(&rendered_rel_path),
        );

        if dest_path.exists() && !dest_path.is_dir() {
            warn!(
                "Skipped generating {:?} because the file already exists",
                dest_path.to_string_lossy().dimmed()
            );
            continue;
        }

        if path.is_dir() {
            fs::create_dir_all(&dest_path)?;
        } else {
            let content = fs::read_to_string(path)?;
            let rendered: String = reg.render_template(&content, data)?;

            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let mut file = File::create(&dest_path)?;
            file.write_all(rendered.as_bytes())?;
        }
    }

    Ok(())
}
