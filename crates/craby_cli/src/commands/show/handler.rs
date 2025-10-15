use std::path::PathBuf;

use craby_codegen::codegen;
use craby_common::config::load_config;
use log::info;
use owo_colors::OwoColorize;

use crate::utils::schema::print_schema;

pub struct ShowOptions {
    pub project_root: PathBuf,
}

pub fn perform(opts: ShowOptions) -> anyhow::Result<()> {
    let config = load_config(&opts.project_root)?;
    let schemas = codegen(craby_codegen::CodegenOptions {
        project_root: &opts.project_root,
        source_dir: &config.source_dir,
    })?;

    let total_mods = schemas.len();
    info!("{} module(s) found\n", total_mods);

    for (i, schema) in schemas.iter().enumerate() {
        println!("{} ({}/{})", schema.module_name.bold(), i + 1, total_mods);
        print_schema(schema)?;
        println!();
    }

    Ok(())
}
