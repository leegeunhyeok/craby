use std::path::PathBuf;

use craby_codegen::{
    constants::GENERATED_COMMENT,
    generators::{
        android_generator::AndroidGenerator, cxx_generator::CxxGenerator,
        ios_generator::IosGenerator, rs_generator::RsGenerator, types::GeneratorInvoker,
    },
    types::{schema::Schema, types::Project},
};
use craby_common::{config::load_config, env::is_initialized};
use log::{debug, info};

use crate::utils::{file::write_file, schema::print_schema};

pub struct CodegenOptions {
    pub project_root: PathBuf,
    pub schemas: Vec<String>,
}

pub fn perform(opts: CodegenOptions) -> anyhow::Result<()> {
    if !is_initialized(&opts.project_root) {
        anyhow::bail!("Craby project is not initialized. Please run `craby init` first.");
    }

    info!("{} module schema(s) found", opts.schemas.len());

    let config = load_config(&opts.project_root)?;
    let schemas = opts
        .schemas
        .iter()
        .enumerate()
        .map(|(i, schema)| {
            let schema = serde_json::from_str::<Schema>(&schema)?;
            info!(
                "Preparing for {} module... ({}/{})",
                schema.module_name,
                i + 1,
                opts.schemas.len()
            );
            print_schema(&schema)?;
            Ok(schema)
        })
        .collect::<Result<Vec<Schema>, anyhow::Error>>()?;

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
