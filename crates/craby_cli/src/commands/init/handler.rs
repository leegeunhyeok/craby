use std::{collections::BTreeMap, path::PathBuf};

use crate::{
    commands::init::{npm::get_latest_version, react_native::setup_react_native_project},
    utils::{
        git::{clone_template, is_git_available},
        template::render_template,
        terminal::with_spinner,
    },
};
use chrono::Datelike;
use craby_build::setup::setup_project;
use craby_codegen::constants::{cxx_mod_cls_name, objc_mod_provider_name};
use craby_common::{
    env::is_rustup_installed,
    utils::string::{flat_case, kebab_case, pascal_case, snake_case},
};
use indoc::formatdoc;
use inquire::{validator::Validation, Text};
use log::{debug, info, warn};
use owo_colors::OwoColorize;

const STATUS_OK: &str = "✓";
const STATUS_WARN: &str = "!";

pub struct InitOptions {
    pub cwd: PathBuf,
    pub pkg_name: String,
}

pub fn perform(opts: InitOptions) -> anyhow::Result<()> {
    let dest_dir = opts.cwd.join(&opts.pkg_name);

    if dest_dir.try_exists()? {
        anyhow::bail!("{} directory already exists", dest_dir.display());
    }

    if !is_git_available() {
        anyhow::bail!("Git command is not available. Please install Git and try again.");
    }

    let non_empty_validator = |input: &str| {
        if input.trim().is_empty() {
            Ok(Validation::Invalid("This field is required.".into()))
        } else {
            Ok(Validation::Valid)
        }
    };

    let email_validator = |input: &str| {
        if email_address::EmailAddress::is_valid(input) {
            Ok(Validation::Valid)
        } else {
            Ok(Validation::Invalid("Invalid email address.".into()))
        }
    };

    let url_validator = |input: &str| {
        if url::Url::parse(input).is_ok() {
            Ok(Validation::Valid)
        } else {
            Ok(Validation::Invalid("Invalid URL.".into()))
        }
    };

    // eg. fast_calculator
    let crate_name = snake_case(&opts.pkg_name);
    let description = Text::new("Enter a description of the package:")
        .with_validator(non_empty_validator)
        .prompt()?;
    let author_name = Text::new("Author name:")
        .with_validator(non_empty_validator)
        .prompt()?;
    let author_email = Text::new("Author email:")
        .with_validator(non_empty_validator)
        .with_validator(email_validator)
        .prompt()?;
    let repository_url = Text::new("Repository URL:")
        .with_validator(non_empty_validator)
        .with_validator(url_validator)
        .prompt()?;

    println!();

    // CxxFastCalculatorModule
    let cxx_name = cxx_mod_cls_name(&crate_name);

    // fastcalculator
    let flat_name = flat_case(&crate_name);

    // fast_calculator
    let snake_name = snake_case(&crate_name);

    // fast-calculator
    let kebab_name = kebab_case(&crate_name);

    // FastCalculator
    let pascal_name = pascal_case(&crate_name);

    // FastCalculatorModuleProvider
    let objc_provider = objc_mod_provider_name(&crate_name);
    let current_year = chrono::Local::now().year().to_string();

    let mut pkg_version = String::new();
    with_spinner(
        "Getting latest package version...",
        |_| match get_latest_version() {
            Ok(res) => {
                pkg_version = res;
                Ok(())
            }
            Err(e) => anyhow::bail!("Failed to get latest package version: {}", e),
        },
    )?;
    info!(
        "{} Found latest package version: {}",
        STATUS_OK.bold().green(),
        pkg_version
    );

    let template_data = BTreeMap::from([
        ("pkg_name", opts.pkg_name.as_str()),
        ("pkg_version", pkg_version.as_str()),
        ("description", description.as_str()),
        ("author_name", author_name.as_str()),
        ("author_email", author_email.as_str()),
        ("repository_url", repository_url.as_str()),
        ("crate_name", crate_name.as_str()),
        ("flat_name", flat_name.as_str()),
        ("snake_name", snake_name.as_str()),
        ("kebab_name", kebab_name.as_str()),
        ("pascal_name", pascal_name.as_str()),
        ("cxx_name", cxx_name.as_str()),
        ("objc_provider", objc_provider.as_str()),
        ("year", current_year.as_str()),
    ]);

    with_spinner("Cloning template...", |_| match clone_template() {
        Ok(template_dir) => {
            debug!(
                "Rendering template... ({:?} -> {:?})",
                template_dir, dest_dir
            );
            render_template(&dest_dir, &template_dir, &template_data)?;
            Ok(())
        }
        Err(e) => anyhow::bail!("Failed to clone template: {}", e),
    })?;
    info!("{} Template generation completed", STATUS_OK.bold().green());

    with_spinner("Setting up React Native project...", |_| {
        if let Err(e) = setup_react_native_project(&dest_dir, &opts.pkg_name) {
            anyhow::bail!("Failed to setup React Native project: {}", e);
        }
        Ok(())
    })?;
    info!(
        "{} React Native project setup completed",
        STATUS_OK.bold().green()
    );

    if is_rustup_installed() {
        with_spinner("Setting up the Rust project, please wait...", |_| {
            if let Err(e) = setup_project() {
                anyhow::bail!("Failed to setup Rust project: {}", e);
            }
            Ok(())
        })?;
        info!("{} Rust project setup completed", STATUS_OK.bold().green());
    } else {
        warn!(
            "{} Please install `rustup` to setup the Rust project for Craby\n\nVisit the Rust website: {}",
            STATUS_WARN.bold().yellow(),
            "https://www.rust-lang.org/tools/install".underline()
        );
    }

    let outro = formatdoc! {
        r#"
        {check_mark} Craby project initialized successfully!

        {get_started}

        {get_started_cmd}

        Run `{codegen_cmd}` to generate Rust code from your native module specifications
        For more information, see the Craby documentation: {docs_url}
        "#,
        check_mark = STATUS_OK.bold().green(),
        get_started = "Get started with your Craby project:".yellow(),
        get_started_cmd = format!("$ cd {} && yarn install", opts.pkg_name).dimmed(),
        codegen_cmd = "npx crabygen".purple().underline(),
        docs_url = "https://craby.rs".dimmed().underline()
    };
    info!("{}", outro);

    Ok(())
}
