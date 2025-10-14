use std::{
    hash::{Hash, Hasher},
    path::PathBuf,
};

use crate::parser::types::{Method, Signal, TypeAnnotation};
use xxhash_rust::xxh3::Xxh3;

pub struct CodegenContext {
    pub name: String,
    pub root: PathBuf,
    pub schemas: Vec<Schema>,
}

#[derive(Debug, Hash)]
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
        let mut hasher = Xxh3::new();
        schemas.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }
}
