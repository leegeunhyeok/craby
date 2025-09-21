use std::{fs, path::PathBuf};

pub fn load_schema_json<T: serde::de::DeserializeOwned>() -> T {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join("schema.json");

    let json_str =
        fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read {}", path.display()));

    serde_json::from_str(&json_str).unwrap_or_else(|_| panic!("Failed to parse {}", path.display()))
}
