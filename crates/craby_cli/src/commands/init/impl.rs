use std::{collections::BTreeMap, path::PathBuf};

use crate::{
    commands::init::validators,
    utils::{template::render_template, terminal::with_spinner},
};
use craby_common::env::is_rustup_installed;
use craby_core::build::setup::setup_project;
use inquire::Text;
use log::{info, warn};
use owo_colors::OwoColorize;

pub struct InitOptions {
    pub project_root: PathBuf,
    pub template_base_path: PathBuf,
    pub library_name: String,
}

pub fn r#impl(opts: InitOptions) -> anyhow::Result<()> {
    let crate_name = Text::new("Enter the crate name")
        .with_default(&opts.library_name)
        .with_validator(validators::CrateNameValidator)
        .prompt()?;

    let root_template = opts.template_base_path.join("root");
    let crates_template = opts.template_base_path.join("crates");
    let data = BTreeMap::from([("crate_name", crate_name.as_str())]);

    render_template(&root_template, &opts.project_root, &data)?;
    render_template(&crates_template, &opts.project_root.join("crates"), &data)?;
    info!("Template generation completed");

    if is_rustup_installed() {
        info!("Setting up the Rust project for Craby 🦀");
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
