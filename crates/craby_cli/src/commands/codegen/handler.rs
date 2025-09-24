use std::{fs, path::PathBuf};

use craby_codegen::{
    constants::GENERATED_COMMENT,
    generators::{
        android_generator::AndroidGenerator, cxx_generator::CxxGenerator,
        ios_generator::IosGenerator, rs_generator::RsGenerator, types::GeneratorInvoker,
    },
    parser::{
        turbo_module_analyzer::parse_schema,
        types::ParseError,
        utils::{render_report, RenderReportOptions},
    },
    types::{schema::Schema, types::Project},
};
use craby_common::{config::load_config, env::is_initialized, utils::fs::collect_files};
use log::{debug, info};

use crate::utils::{file::write_file, schema::print_schema};

pub struct CodegenOptions {
    pub project_root: PathBuf,
}

pub fn perform(opts: CodegenOptions) -> anyhow::Result<()> {
    if !is_initialized(&opts.project_root) {
        anyhow::bail!("Craby project is not initialized. Please run `craby init` first.");
    }

    let config = load_config(&opts.project_root)?;

    info!(
        "Collecting source files from {}",
        config.source_dir.display()
    );
    let srcs = collect_files(&config.source_dir, &|path: &PathBuf| {
        path.extension().unwrap_or_default() == "ts"
            && path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .starts_with("Native")
    })?;
    debug!("{} source file(s) found", srcs.len());

    if srcs.len() == 0 {
        anyhow::bail!("No native module specification files found.");
    }

    srcs.iter()
        .try_for_each(|path| -> Result<(), anyhow::Error> {
            let src = fs::read_to_string(path)?;
            let src = src.as_str();

            match parse_schema(src) {
                Ok(schemas) => {
                    info!("{} module schema(s) found", schemas.len());
                }
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

            Ok(())
        })?;

    let schemas: Vec<Schema> = vec![]; // TODO
    info!("{} module schema(s) found", schemas.len());

    // let schemas = opts
    //     .schemas
    //     .iter()
    //     .enumerate()
    //     .map(|(i, schema)| {
    //         let schema = serde_json::from_str::<Schema>(&schema)?;
    //         info!(
    //             "Preparing for {} module... ({}/{})",
    //             schema.module_name,
    //             i + 1,
    //             opts.schemas.len()
    //         );
    //         print_schema(&schema)?;
    //         Ok(schema)
    //     })
    //     .collect::<Result<Vec<Schema>, anyhow::Error>>()?;

    let project = Project {
        name: config.project.name,
        root: opts.project_root,
        schemas,
    };

    let mut generate_res = vec![];
    let generators: Vec<Box<dyn GeneratorInvoker>> = vec![
        Box::new(AndroidGenerator::new()),
        Box::new(IosGenerator::new()),
        Box::new(RsGenerator::new()),
        Box::new(CxxGenerator::new()),
    ];

    info!("Generating files...");
    generators
        .iter()
        .try_for_each(|generator| -> Result<(), anyhow::Error> {
            generate_res.extend(generator.invoke_generate(&project)?);
            Ok(())
        })?;

    let mut wrote_cnt = 0;
    generate_res
        .iter()
        .try_for_each(|res| -> Result<(), anyhow::Error> {
            let content = if res.overwrite {
                with_generated_comment(&res.path, &res.content)
            } else {
                without_generated_comment(&res.content)
            };
            let write = write_file(&res.path, &content, res.overwrite)?;

            if write {
                wrote_cnt += 1;
                debug!("File generated: {}", res.path.display());
            } else {
                debug!("Skipped writing to {}", res.path.display());
            }

            Ok(())
        })?;

    info!("{} files generated", wrote_cnt);
    info!("Codegen completed successfully 🎉");

    Ok(())
}

fn with_generated_comment(path: &PathBuf, code: &String) -> String {
    match path.extension() {
        Some(ext) => match ext.to_str().unwrap() {
            // Source files
            "rs" | "cpp" | "hpp" | "mm" => format!("// {}\n{}\n", GENERATED_COMMENT, code),
            // CMakeLists.txt
            "txt" => format!("# {}\n{}\n", GENERATED_COMMENT, code),
            _ => without_generated_comment(code),
        },
        None => without_generated_comment(code),
    }
}

fn without_generated_comment(code: &String) -> String {
    format!("{}\n", code)
}
