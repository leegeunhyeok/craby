use std::path::PathBuf;

use craby_codegen::{
    constants::{cxx_mod_cls_name, GENERATED_COMMENT},
    generator::CodeGenerator,
    generators::{rs_generator::RustGenerator, types::Generator},
    platform::cxx,
    types::schema::Schema,
};
use craby_common::{
    config::load_config,
    constants::{crate_dir, cxx_dir},
    env::is_initialized,
};
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

    let config = load_config(&opts.project_root)?;
    let crate_path = crate_dir(&opts.project_root);
    let crate_src_path = crate_path.join("src");
    let cxx_dir = cxx_dir(&opts.project_root);
    let cxx_mod_cls_name = cxx_mod_cls_name(&config.project.name);

    info!("{} module schema(s) found", opts.schemas.len());

    let mut generate_res = vec![];
    let generator = CodeGenerator::new();
    let total_mods = opts.schemas.len();
    let mut codegen_res = vec![];

    let generators = [RustGenerator::new()];

    let schemas = opts
        .schemas
        .iter()
        .map(|schema| serde_json::from_str::<Schema>(&schema))
        .collect::<Result<Vec<Schema>, serde_json::Error>>()?;

    schemas
        .iter()
        .try_for_each(|schema| -> Result<(), anyhow::Error> {
            print_schema(schema)?;
            Ok(())
        })?;

    generators
        .iter()
        .try_for_each(|generator| -> Result<(), anyhow::Error> {
            generate_res.extend(generator.generate(&opts.project_root, &schemas)?);
            Ok(())
        })?;

    generate_res
        .iter()
        .try_for_each(|res| -> Result<(), anyhow::Error> {
            let content = with_generated_comment(&res.content);
            let write = write_file(&res.path, &content, res.overwrite)?;

            if write {
                info!("File generated: {:#?}", res.path);
            } else {
                debug!("Skipped writing to {:#?}", res.path);
            }

            Ok(())
        })?;

    opts.schemas
        .iter()
        .enumerate()
        .try_for_each(|(i, schema)| -> Result<(), anyhow::Error> {
            let schema = serde_json::from_str::<Schema>(&schema)?;
            println!(
                "Generating for {} module... ({}/{})",
                schema.module_name,
                i + 1,
                total_mods
            );

            if schema.r#type == "Component" {
                return Err(anyhow::anyhow!("Component type is not supported"));
            }

            let res = generator.generate(&schema)?;

            codegen_res.push(res);

            Ok(())
        })?;

    write_file(
        &cxx_dir.join(format!("{}.cpp", cxx_mod_cls_name)),
        &with_generated_comment(&cxx::template::mod_cxx(&codegen_res)),
        true,
    )?;
    write_file(
        &cxx_dir.join(format!("{}.hpp", cxx_mod_cls_name)),
        &with_generated_comment(&cxx::template::mod_cxx_h(&codegen_res)),
        true,
    )?;
    write_file(
        &cxx_dir.join("bridging-generated.hpp"),
        &with_generated_comment(&cxx::template::cxx_bridging_h(&codegen_res)),
        true,
    )?;

    info!("Codegen completed successfully 🎉");

    Ok(())
}

fn with_generated_comment(code: &String) -> String {
    format!("// {}\n{}\n", GENERATED_COMMENT, code)
}
