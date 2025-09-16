use std::path::PathBuf;

use craby_codegen::types::schema::Schema;
use log::info;
use owo_colors::OwoColorize;

use crate::utils::schema::print_schema;

pub struct ShowOptions {
    pub project_root: PathBuf,
    pub package_name: String,
    pub schemas: Vec<String>,
}

pub fn perform(opts: ShowOptions) -> anyhow::Result<()> {
    let total_mods = opts.schemas.len();
    info!(
        "{} module(s) found in {} package\n",
        total_mods, opts.package_name
    );

    for (i, schema) in opts.schemas.iter().enumerate() {
        let schema = serde_json::from_str::<Schema>(&schema)?;
        println!("{} ({}/{})", schema.module_name.bold(), i + 1, total_mods);
        print_schema(&schema)?;
    }

    Ok(())
}
