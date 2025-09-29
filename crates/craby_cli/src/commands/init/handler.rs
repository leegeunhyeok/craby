use std::{collections::BTreeMap, path::PathBuf};

use crate::{
    commands::init::validators,
    utils::{
        git::{clone_template, is_git_available},
        template::render_template,
        terminal::with_spinner,
    },
};
use craby_build::setup::setup_project;
use craby_codegen::constants::{cxx_mod_cls_name, objc_mod_provider_name};
use craby_common::{
    env::is_rustup_installed,
    utils::string::{flat_case, kebab_case, snake_case},
};
use inquire::Text;
use log::{info, warn};
use owo_colors::OwoColorize;

pub struct InitOptions {
    pub project_root: PathBuf,
    pub template_base_path: PathBuf,
    pub package_name: String,
}

pub fn perform(opts: InitOptions) -> anyhow::Result<()> {
    if is_git_available() == false {
        anyhow::bail!("Git command is not available. Please install Git and try again.");
    }

    // eg. fast_calculator
    let crate_name = snake_case(&opts.package_name);
    let crate_name = Text::new("Enter the crate name")
        .with_default(&crate_name)
        .with_validator(validators::CrateNameValidator)
        .prompt()?;

    // CxxFastCalculatorModule
    let cxx_name = cxx_mod_cls_name(&crate_name);

    // fastcalculator
    let flat_name = flat_case(&crate_name);

    // fast-calculator
    let kebab_name = kebab_case(&crate_name);

    // FastCalculatorModuleProvider
    let objc_provider_name = objc_mod_provider_name(&crate_name);

    let root_template = opts.template_base_path.join("root");
    let crates_template = opts.template_base_path.join("crates");
    let cxx_template = opts.template_base_path.join("cpp");
    let android_template = opts.template_base_path.join("android");
    let ios_template = opts.template_base_path.join("ios");
    let template_data = BTreeMap::from([
        ("crate_name", crate_name.as_str()),
        ("flat_name", flat_name.as_str()),
        ("kebab_name", kebab_name.as_str()),
        ("cxx_name", cxx_name.as_str()),
        ("objc_provider_name", objc_provider_name.as_str()),
    ]);

    let template_dir = clone_template()?;
    render_template(&template_dir, &template_data)?;
    info!("Template generation completed");

    if is_rustup_installed() {
        info!("Setting up the Rust project");
        with_spinner("Setting up the project, please wait...", |_| {
            setup_project()?;
            Ok(())
        })?;
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
