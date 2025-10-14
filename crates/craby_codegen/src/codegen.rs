use std::{fs, path::PathBuf};

use craby_common::{constants::SPEC_FILE_PREFIX, utils::fs::collect_files};
use log::debug;

use crate::{
    parser::{
        native_spec_parser::try_parse_schema,
        types::ParseError,
        utils::{render_report, RenderReportOptions},
    },
    types::Schema,
};

pub struct CodegenOptions<'a> {
    pub project_root: &'a PathBuf,
    pub source_dir: &'a PathBuf,
}

pub fn codegen<'a>(opts: CodegenOptions<'a>) -> Result<Vec<Schema>, anyhow::Error> {
    let srcs = collect_files(&opts.source_dir, &|path: &PathBuf| {
        path.extension().unwrap_or_default() == "ts"
            && path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .starts_with(SPEC_FILE_PREFIX)
    })?;
    debug!("{} source file(s) found", srcs.len());

    if srcs.len() == 0 {
        anyhow::bail!("No native module specification files found.");
    }

    let collected_schemas = srcs
        .iter()
        .map(|path| {
            let src = fs::read_to_string(path)?;
            let src = src.as_str();

            match try_parse_schema(src) {
                Ok(schemas) => Ok(schemas),
                Err(ParseError::Oxc { diagnostics }) => {
                    render_report(
                        diagnostics,
                        RenderReportOptions {
                            project_root: &opts.project_root,
                            path,
                            src,
                        },
                    );
                    anyhow::bail!("Failed to parse schema");
                }
                Err(ParseError::General(e)) => {
                    anyhow::bail!(e);
                }
            }
        })
        .collect::<Result<Vec<Vec<Schema>>, anyhow::Error>>()?;

    let mut schemas = collected_schemas.into_iter().flatten().collect::<Vec<_>>();
    schemas.sort_by_key(|v| v.module_name.to_lowercase());

    debug!("Collected schemas: {:?}", schemas);

    Ok(schemas)
}
