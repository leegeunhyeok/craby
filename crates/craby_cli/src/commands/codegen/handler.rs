use std::path::PathBuf;

use craby_codegen::{
    constants::{cxx_mod_cls_name, GENERATED_COMMENT},
    generator::CodeGenerator,
    platform::{cxx, rust},
    types::schema::Schema,
};
use craby_common::{
    config::load_config,
    constants::{crate_dir, cxx_dir, impl_mod_name},
    env::is_initialized,
};
use log::info;

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

    let generator = CodeGenerator::new();
    let total_mods = opts.schemas.len();
    let mut codegen_res = vec![];

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

            print_schema(&schema)?;
            write_file(
                crate_src_path.join(format!("{}.rs", impl_mod_name(&schema.module_name))),
                format!("{}\n", res.impl_code),
                false,
            )?;

            codegen_res.push(res);

            Ok(())
        })?;

    write_file(
        crate_src_path.join("lib.rs"),
        with_generated_comment(rust::template::lib_rs(&codegen_res)),
        true,
    )?;
    write_file(
        crate_src_path.join("ffi.rs"),
        with_generated_comment(rust::template::ffi_rs(&codegen_res)),
        true,
    )?;
    write_file(
        crate_src_path.join("types.rs"),
        with_generated_comment(rust::template::types_rs()),
        true,
    )?;
    write_file(
        crate_src_path.join("generated.rs"),
        with_generated_comment(rust::template::generated_rs(&codegen_res)),
        true,
    )?;
    write_file(
        cxx_dir.join(format!("{}.cpp", cxx_mod_cls_name)),
        with_generated_comment(cxx::template::mod_cxx(&codegen_res)),
        true,
    )?;
    write_file(
        cxx_dir.join(format!("{}.hpp", cxx_mod_cls_name)),
        with_generated_comment(cxx::template::mod_cxx_h(&codegen_res)),
        true,
    )?;
    write_file(
        cxx_dir.join("bridging-generated.hpp"),
        with_generated_comment(cxx::template::cxx_bridging_h(&codegen_res)),
        true,
    )?;

    info!("Codegen completed successfully 🎉");

    Ok(())
}

fn with_generated_comment(code: String) -> String {
    format!("// {}\n{}\n", GENERATED_COMMENT, code)
}
