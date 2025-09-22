use std::path::PathBuf;

use craby_codegen::types::schema::Schema;
use log::info;
use owo_colors::OwoColorize;

use crate::utils::schema::print_schema;

pub struct ShowOptions {
    pub project_root: PathBuf,
}

pub fn perform(opts: ShowOptions) -> anyhow::Result<()> {
    let schemas: Vec<String> = vec![]; // TODO

    let total_mods = schemas.len();
    info!("{} module(s) found\n", total_mods);

    // for (i, schema) in opts.schemas.iter().enumerate() {
    //     let schema = serde_json::from_str::<Schema>(&schema)?;
    //     println!("{} ({}/{})", schema.module_name.bold(), i + 1, total_mods);
    //     print_schema(&schema)?;
    // }

    Ok(())
}
