use std::path::PathBuf;

use craby_codegen::{
    constants::GENERATED_COMMENT,
    generators::{
        android_generator::AndroidGenerator, cxx_generator::CxxGenerator,
        ios_generator::IosGenerator, rs_generator::RsGenerator, types::GeneratorInvoker,
    },
    types::schema::Schema,
};
use craby_common::env::is_initialized;
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

    let mut generate_res = vec![];
    let total_mods = opts.schemas.len();
    let generators: Vec<Box<dyn GeneratorInvoker>> = vec![
        Box::new(AndroidGenerator::new()),
        Box::new(IosGenerator::new()),
        Box::new(RsGenerator::new()),
        Box::new(CxxGenerator::new()),
    ];

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
                total_mods
            );
            print_schema(&schema)?;
            Ok(schema)
        })
        .collect::<Result<Vec<Schema>, anyhow::Error>>()?;

    info!("Generating files...");
    generators
        .iter()
        .try_for_each(|generator| -> Result<(), anyhow::Error> {
            generate_res.extend(generator.invoke_generate(&opts.project_root, &schemas)?);
            Ok(())
        })?;

    let mut wrote_cnt = 0;
    generate_res
        .iter()
        .try_for_each(|res| -> Result<(), anyhow::Error> {
            let content = with_generated_comment(&res.path, &res.content);
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
            _ => format!("{}\n", code),
        },
        None => format!("{}\n", code),
    }
}
