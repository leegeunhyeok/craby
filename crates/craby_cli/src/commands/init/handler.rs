use std::{collections::BTreeMap, fs, path::PathBuf};

use crate::{
    commands::init::validators,
    utils::{template::render_template, terminal::with_spinner},
};
use craby_build::setup::setup_project;
use craby_codegen::{
    constants::{cxx_mod_cls_name, objc_mod_provider_name},
    types::schema::Schema,
};
use craby_common::{
    env::is_rustup_installed,
    utils::string::{flat_case, kebab_case, snake_case},
};
use inquire::Text;
use log::{debug, info, warn};
use owo_colors::OwoColorize;

pub struct InitOptions {
    pub project_root: PathBuf,
    pub template_base_path: PathBuf,
    pub package_name: String,
    pub schemas: Vec<String>,
}

pub fn perform(opts: InitOptions) -> anyhow::Result<()> {
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
    let data = BTreeMap::from([
        ("crate_name", crate_name.as_str()),
        ("flat_name", flat_name.as_str()),
        ("kebab_name", kebab_name.as_str()),
        ("cxx_name", cxx_name.as_str()),
        ("objc_provider_name", objc_provider_name.as_str()),
    ]);

    fs::create_dir_all(opts.project_root.join(".craby"))?;
    render_template(&root_template, &opts.project_root, &data)?;
    render_template(&crates_template, &opts.project_root.join("crates"), &data)?;
    render_template(&android_template, &opts.project_root.join("android"), &data)?;
    render_template(&ios_template, &opts.project_root.join("ios"), &data)?;

    // Generate C++ code for each TurboModule schema
    opts.schemas.into_iter().try_for_each(|schema| {
        let schema = serde_json::from_str::<Schema>(&schema)?;
        let turbo_module_name = schema.module_name.clone();
        let mut cxx_template_data = data.clone();
        cxx_template_data.insert("turbo_module_name", turbo_module_name.as_str());
        render_template(
            &cxx_template,
            &opts.project_root.join("cpp"),
            &cxx_template_data,
        )?;
        Ok::<(), anyhow::Error>(())
    })?;

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
        fs::write(&gitignore, "# Craby\n.craby\ntarget/\n")?;
    }

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
