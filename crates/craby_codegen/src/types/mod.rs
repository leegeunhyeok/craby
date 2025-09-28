use std::path::PathBuf;

use crate::parser::types::{Signal, Method, TypeAnnotation};

pub struct CodegenContext {
    pub name: String,
    pub root: PathBuf,
    pub schemas: Vec<Schema>,
}

#[derive(Debug)]
pub struct Schema {
    pub module_name: String,
    // `TypeAnnotation::ObjectTypeAnnotation`
    pub aliases: Vec<TypeAnnotation>,
    // `TypeAnnotation::EnumTypeAnnotation`
    pub enums: Vec<TypeAnnotation>,
    pub methods: Vec<Method>,
    pub signals: Vec<Signal>,
}
