use std::{
    path::{Path, PathBuf},
    time::Instant,
};

use craby_codegen::{
    codegen,
    constants::GENERATED_COMMENT,
    generators::{
        android_generator::AndroidGenerator,
        cxx_generator::CxxGenerator,
        ios_generator::IosGenerator,
        rs_generator::RsGenerator,
        types::{Generator, GeneratorInvoker},
    },
    types::CodegenContext,
};
use craby_common::{config::load_config, constants::craby_tmp_dir, env::is_initialized};
use log::{debug, info};
use owo_colors::OwoColorize;

use crate::utils::{file::write_file, schema::print_schema};

#[derive(Debug)]
pub struct CodegenOptions {
    pub project_root: PathBuf,
    pub overwrite: bool,
}

pub fn perform(opts: CodegenOptions) -> anyhow::Result<()> {
    if !is_initialized(&opts.project_root) {
        anyhow::bail!("Craby project is not initialized. Please run `craby init` first.");
    }

    let tmp_dir = craby_tmp_dir(&opts.project_root);
    let config = load_config(&opts.project_root)?;
    let start_time = Instant::now();

    debug!("Options: {:?}", opts);
    info!(
        "Collecting source files... {}",
        format!("({})", config.source_dir.display()).dimmed()
    );
    let schemas = codegen(craby_codegen::CodegenOptions {
        project_root: &opts.project_root,
        source_dir: &config.source_dir,
    })?;
    let total_schemas = schemas.len();
    info!("{} module schema(s) found", total_schemas);

    // Print schema for each module
    for (i, schema) in schemas.iter().enumerate() {
        info!(
            "Found module: {} ({}/{})",
            schema.module_name,
            i + 1,
            total_schemas,
        );
        print_schema(schema)?;
        println!();
    }

    let ctx = CodegenContext {
        project_name: config.project.name,
        root: opts.project_root.clone(),
        schemas,
        android_package_name: config.android.package_name,
    };

    debug!("Cleaning up...");
    AndroidGenerator::cleanup(&ctx)?;
    IosGenerator::cleanup(&ctx)?;
    RsGenerator::cleanup(&ctx)?;
    CxxGenerator::cleanup(&ctx)?;

    let mut generate_res = vec![];
    let generators: Vec<Box<dyn GeneratorInvoker>> = vec![
        Box::new(AndroidGenerator::new()),
        Box::new(IosGenerator::new()),
        Box::new(RsGenerator::new()),
        Box::new(CxxGenerator::new()),
    ];

    info!("Generating files...");
    for generator in generators {
        generate_res.extend(generator.invoke_generate(&ctx)?);
    }

    let mut generated_cnt = 0;
    let mut preserved_files = vec![];
    for res in generate_res {
        let content = if res.overwrite {
            with_generated_comment(&res.path, &res.content)
        } else {
            without_generated_comment(&res.content)
        };

        let should_overwrite = opts.overwrite && res.overwrite;
        if write_file(&res.path, &content, should_overwrite)? {
            generated_cnt += 1;
            debug!("File generated: {}", res.path.display());
        } else {
            // Save the content to a temporary directory if it's not written
            let file_name = res.path.file_name().unwrap();
            let dest = tmp_dir.join(file_name);
            debug!("Saving to temporary directory: {}", dest.display());
            write_file(&dest, &content, true)?;

            if res.overwrite {
                preserved_files.push(
                    res.path
                        .strip_prefix(&opts.project_root)?
                        .to_string_lossy()
                        .to_string(),
                );
            }
        }
    }

    let elapsed = start_time.elapsed().as_millis();
    info!("{} files generated", generated_cnt);

    let preserved_file_cnt = preserved_files.len();
    if preserved_file_cnt > 0 {
        info!("Preserving existing files");

        for (idx, file) in preserved_files.iter().enumerate() {
            let line = if idx == preserved_file_cnt - 1 {
                "└─"
            } else {
                "├─"
            };
            println!("{} {}", line, file.dimmed());
        }
    }

    info!(
        "Codegen completed successfully 🎉 {}",
        format!("({}ms)", elapsed).dimmed()
    );

    Ok(())
}

fn with_generated_comment(path: &Path, code: &str) -> String {
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

fn without_generated_comment(code: &str) -> String {
    format!("{}\n", code)
}
