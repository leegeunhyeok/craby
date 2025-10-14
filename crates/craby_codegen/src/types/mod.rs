use std::{hash::Hasher, path::PathBuf};

use crate::parser::types::{Method, Signal, TypeAnnotation};
use log::debug;
use serde::Serialize;
use xxhash_rust::xxh3::Xxh3;

pub struct CodegenContext {
    pub name: String,
    pub root: PathBuf,
    pub schemas: Vec<Schema>,
}

#[derive(Debug, Serialize)]
pub struct Schema {
    pub module_name: String,
    // `TypeAnnotation::ObjectTypeAnnotation`
    pub aliases: Vec<TypeAnnotation>,
    // `TypeAnnotation::EnumTypeAnnotation`
    pub enums: Vec<TypeAnnotation>,
    pub methods: Vec<Method>,
    pub signals: Vec<Signal>,
}

impl Schema {
    pub fn to_hash(schemas: &Vec<Schema>) -> String {
        let serialized = serde_json::to_string(schemas).unwrap();
        debug!("Serialized schemas: {}", serialized);
        let mut hasher = Xxh3::new();
        hasher.write(serialized.as_bytes());
        format!("{:016x}", hasher.finish())
    }
}
