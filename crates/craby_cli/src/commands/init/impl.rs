use std::{collections::BTreeMap, fs, path::PathBuf};

use crate::{
    commands::init::validators,
    utils::{template::render_template, terminal::with_spinner},
};
use craby_common::{build::setup::setup_project, env::is_rustup_installed, utils::sanitize_str};
use inquire::Text;
use log::{debug, info, warn};
use owo_colors::OwoColorize;

pub struct InitOptions {
    pub project_root: PathBuf,
    pub template_base_path: PathBuf,
    pub lib_name: String,
}

pub fn r#impl(opts: InitOptions) -> anyhow::Result<()> {
    let crate_name = Text::new("Enter the crate name")
        .with_default(&sanitize_str(&opts.lib_name))
        .with_validator(validators::CrateNameValidator)
        .prompt()?;
    let lib_name = crate_name.replace("_", "");

    let root_template = opts.template_base_path.join("root");
    let crates_template = opts.template_base_path.join("crates");
    let data = BTreeMap::from([
        ("crate_name", crate_name.as_str()),
        ("lib_name", lib_name.as_str()),
    ]);

    fs::create_dir_all(opts.project_root.join(".craby"))?;
    render_template(&root_template, &opts.project_root, &data)?;
    render_template(&crates_template, &opts.project_root.join("crates"), &data)?;
    info!("Template generation completed");

    let gitignore = opts.project_root.join(".gitignore");
    if gitignore.exists() {
        let content = fs::read_to_string(&gitignore)?;
        let mut append_contents = vec![];

        if !content.contains("target/") {
            append_contents.push("target/".to_string());
        }

        if !content.contains(".craby") {
            append_contents.push(".craby".to_string());
            debug!("`.craby` directory added to .gitignore");
        }

        if append_contents.len() > 0 {
            debug!("{} added to .gitignore", append_contents.join(", "));
            fs::write(
                &gitignore,
                format!("{}\n\n# Craby\n{}", content, append_contents.join("\n")),
            )?;
        }
    } else {
        fs::write(&gitignore, "# Craby\n.craby\ntarget/")?;
    }

    if is_rustup_installed() {
        info!("Setting up the Rust project");
        with_spinner("Setting up the project, please wait...", setup_project)?;
        info!("Rust project setup completed");
    } else {
        warn!(
            "Please install Rustup to setup the Rust project for Craby\n\nVisit the Rust website: {}",
            "https://www.rust-lang.org/tools/install".underline()
        );
    }

    info!(
        "Craby project initialized successfully 🎉\n\nRun `{}` to generate Rust code from your TurboModule specifications",
        "craby codegen".green().underline()
    );

    Ok(())
}
