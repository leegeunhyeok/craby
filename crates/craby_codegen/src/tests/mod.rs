use std::{fs, path::PathBuf};
use crate::{
    generator::CodeGenerator,
    types::{schema::Schema, types::CodegenResult},
};

pub fn load_schema_json<T: serde::de::DeserializeOwned>() -> T {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join("schema.json");

    let json_str =
        fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read {}", path.display()));

    serde_json::from_str(&json_str).unwrap_or_else(|_| panic!("Failed to parse {}", path.display()))
}

pub fn load_schema_as_codegen_res() -> CodegenResult {
    let schema = load_schema_json::<Schema>();
    let generator = CodeGenerator::new();
    generator.generate(&schema).unwrap()
}
