use std::{collections::BTreeMap, path::Path};

use chrono::Datelike;
use craby_codegen::types::{CxxModuleName, ObjCProviderName};
use craby_common::utils::string::{flat_case, kebab_case, pascal_case, snake_case};
use inquire::{validator::Validation, Text};
use log::debug;

use crate::utils::{
    git::clone_template,
    log::success,
    template::{render_template, TemplateData},
    terminal::with_spinner,
};

pub fn prompt_for_template_data(pkg_name: &str) -> anyhow::Result<TemplateData> {
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
    let crate_name = snake_case(pkg_name);
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
    let cxx_name = CxxModuleName::from(&crate_name);

    // fastcalculator
    let flat_name = flat_case(&crate_name);

    // fast_calculator
    let snake_name = snake_case(&crate_name);

    // fast-calculator
    let kebab_name = kebab_case(&crate_name);

    // FastCalculator
    let pascal_name = pascal_case(&crate_name);

    // FastCalculatorModuleProvider
    let objc_provider = ObjCProviderName::from(&crate_name);

    let current_year = chrono::Local::now().year();

    let template_data = BTreeMap::from([
        ("pkg_name", pkg_name.to_string()),
        ("description", description),
        ("author_name", author_name),
        ("author_email", author_email),
        ("repository_url", repository_url),
        ("crate_name", crate_name),
        ("flat_name", flat_name),
        ("snake_name", snake_name),
        ("kebab_name", kebab_name),
        ("pascal_name", pascal_name),
        ("cxx_name", cxx_name.to_string()),
        ("objc_provider", objc_provider.to_string()),
        ("year", current_year.to_string()),
        ("pkg_version", format!("^{}", env!("CARGO_PKG_VERSION"))),
    ]);

    Ok(template_data)
}

pub fn setup_template(dest_dir: &Path, template_data: &TemplateData) -> anyhow::Result<()> {
    with_spinner("Cloning template...", |_| match clone_template() {
        Ok(template_dir) => setup_template_impl(dest_dir, &template_dir, template_data),
        Err(e) => anyhow::bail!("Failed to clone template: {}", e),
    })?;
    success("Template generation completed");

    Ok(())
}

pub fn setup_template_impl(
    dest_dir: &Path,
    template_dir: &Path,
    template_data: &TemplateData,
) -> anyhow::Result<()> {
    debug!(
        "Rendering template... ({:?} -> {:?})",
        template_dir, dest_dir
    );
    render_template(dest_dir, template_dir, template_data)?;

    Ok(())
}
