use std::{fs, path::Path};

use craby_codegen::types::Schema;
use craby_common::constants::{crate_dir, HASH_COMMAND_PREFIX};
use log::debug;

/// Validate the schema(s) by comparing the hash in the `generated.rs` file
///
/// If the hash does not match, it will bail with an error
/// In this case, you need to re-generate the `generated.rs` file by `codegen` command
pub fn validate_schema(project_root: &Path, schemas: &[Schema]) -> anyhow::Result<()> {
    let src = fs::read_to_string(crate_dir(project_root).join("src").join("generated.rs"))?;
    let src_hash = get_hash_from_src(&src)?;
    let curr_hash = Schema::to_hash(schemas);
    debug!("Current hash: {}, Expected hash: {}, ", curr_hash, src_hash);
    if src_hash != curr_hash {
        anyhow::bail!("Generated hash does not match the hash in the `generated.rs` file (current: {}, expected: {})", curr_hash, src_hash);
    }
    Ok(())
}

/// Get the hash from the `generated.rs` file
///
/// # Example
///
/// ```rust,ignore
/// // Hash: xxx
/// ```
///
/// # Returns
///
/// The hash string (eg. `xxx`)
fn get_hash_from_src(src: &str) -> Result<String, anyhow::Error> {
    let hash = src
        .lines()
        .find(|line| line.starts_with(HASH_COMMAND_PREFIX))
        .unwrap();

    match hash.split(HASH_COMMAND_PREFIX).nth(1) {
        Some(hash) => Ok(hash.trim().to_string()),
        None => anyhow::bail!("Hash not found in the `generated.rs` file"),
    }
}
