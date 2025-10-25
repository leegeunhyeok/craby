use std::path::PathBuf;

use crate::{
    commands::init::{
        prepare::validate_env,
        react_native::setup_react_native_project,
        rust::setup_rust_toolchain,
        template::{prompt_for_template_data, setup_template},
    },
    utils::log::{sym, Status},
};
use indoc::formatdoc;
use log::info;
use owo_colors::OwoColorize;

pub struct InitOptions {
    pub cwd: PathBuf,
    pub pkg_name: String,
}

pub fn perform(opts: InitOptions) -> anyhow::Result<()> {
    let dest_dir = opts.cwd.join(&opts.pkg_name);
    validate_env(&dest_dir)?;

    let template_data = prompt_for_template_data(&opts.pkg_name)?;
    setup_template(&dest_dir, &template_data)?;
    setup_react_native_project(&dest_dir, &opts.pkg_name)?;
    setup_rust_toolchain()?;

    let outro = formatdoc! {
        r#"
        {check_mark} Craby project initialized successfully!

        {get_started}

        {get_started_cmd}

        Run `{codegen_cmd}` to generate Rust code from your native module specifications
        For more information, see the Craby documentation: {docs_url}
        "#,
        check_mark = sym(Status::Ok),
        get_started = "Get started with your Craby project:".yellow(),
        get_started_cmd = format!("$ cd {} && yarn install", opts.pkg_name).dimmed(),
        codegen_cmd = "npx crabygen".purple().underline(),
        docs_url = "https://craby.rs".dimmed().underline()
    };
    info!("{}", outro);

    Ok(())
}
