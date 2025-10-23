use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use crate::utils::{
    git::{clone_template, is_git_available},
    template::render_template,
    terminal::{run_command, with_spinner},
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

    let template_data = BTreeMap::from([
        ("pkg_name", opts.pkg_name.as_str()),
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

    with_spinner("Cloning template...", |_| {
        let template_dir = clone_template()?;
        debug!(
            "Rendering template... ({:?} -> {:?})",
            template_dir, dest_dir
        );
        render_template(&dest_dir, &template_dir, &template_data)?;
        Ok(())
    })?;
    info!("{} Template generation completed", STATUS_OK.bold().green());

    setup_react_native_project(&dest_dir, &opts.pkg_name)?;

    if is_rustup_installed() {
        with_spinner("Setting up the Rust project, please wait...", |_| {
            setup_project()?;
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

    info!(
        "Craby project initialized successfully\n\nRun `{}` to generate Rust code from your native module specifications",
        "npx crabygen".green().underline()
    );

    Ok(())
}

fn setup_react_native_project(dest_dir: &Path, pkg_name: &str) -> anyhow::Result<()> {
    let app_name = pascal_case(pkg_name);

    // Root package.json
    let root_package_json_path = dest_dir.join("package.json");
    let raw_package_json: String = fs::read_to_string(&root_package_json_path)?;
    let mut package_json = serde_json::from_str::<serde_json::Value>(&raw_package_json)?;
    if let Some(obj) = package_json.as_object_mut() {
        debug!("Inserting workspaces field");
        obj.insert("workspaces".to_string(), serde_json::json!(["example"]));

        fs::write(
            root_package_json_path,
            serde_json::to_string_pretty(&package_json)?,
        )?;
    }

    run_command(
        "npx",
        &[
            "@react-native-community/cli@latest",
            "init",
            app_name.as_str(),
            "--skip-install",
            "--skip-git-init",
        ],
        Some(&dest_dir.to_string_lossy()),
    )?;

    let react_native_dir = dest_dir.join(&app_name);
    let react_native_package_json_path = react_native_dir.join("package.json");
    let raw_package_json = fs::read_to_string(&react_native_package_json_path)?;
    let mut package_json = serde_json::from_str::<serde_json::Value>(&raw_package_json)?;
    if let Some(obj) = package_json.as_object_mut() {
        if let Some(dependencies) = obj.get_mut("dependencies") {
            if let Some(dependencies_obj) = dependencies.as_object_mut() {
                debug!("Inserting dependencies");
                dependencies_obj.insert(pkg_name.to_string(), serde_json::json!("workspace:*"));
            }
        }

        if let Some(dev_dependencies) = obj.get_mut("devDependencies") {
            if let Some(dev_dependencies_obj) = dev_dependencies.as_object_mut() {
                debug!("Inserting devDependencies");
                dev_dependencies_obj.insert("@craby/devkit".to_string(), serde_json::json!("*"));
            }
        }

        fs::write(
            react_native_package_json_path,
            serde_json::to_string_pretty(&package_json)?,
        )?;
    }

    let metro_config = formatdoc! {
        r#"
        const {{ getMetroConfig }} = require('@craby/devkit');
        const {{ getDefaultConfig, mergeConfig }} = require('@react-native/metro-config');

        /**
         * Metro configuration
         * https://reactnative.dev/docs/metro
         *
         * @type {{import('@react-native/metro-config').MetroConfig}}
         */
        const config = getMetroConfig(__dirname);

        module.exports = mergeConfig(getDefaultConfig(__dirname), config);
        "#
    };

    let react_native_config = formatdoc! {
        r#"
        const path = require('node:path');
        const {{ withWorkspaceModule }} = require('@craby/devkit');

        const modulePackagePath = path.resolve(__dirname, '..');
        const config = {{}};

        module.exports = withWorkspaceModule(config, modulePackagePath);
        "#
    };

    debug!("Overwriting config files");
    fs::write(react_native_dir.join("metro.config.js"), metro_config)?;
    fs::write(
        react_native_dir.join("react-native.config.js"),
        react_native_config,
    )?;

    if react_native_dir.try_exists()? {
        debug!(
            "Renaming React Native project to example: {:?}",
            react_native_dir
        );
        fs::rename(react_native_dir, dest_dir.join("example"))?;
    }

    Ok(())
}
