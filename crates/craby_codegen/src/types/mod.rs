use std::{fmt::Display, hash::Hasher, path::PathBuf};

use crate::parser::types::{Method, Signal, TypeAnnotation};
use craby_common::utils::string::{flat_case, pascal_case};
use log::debug;
use serde::Serialize;
use xxhash_rust::xxh3::Xxh3;

pub struct CodegenContext {
    pub project_name: String,
    pub root: PathBuf,
    pub schemas: Vec<Schema>,
    pub android_package_name: String,
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
    pub fn to_hash(schemas: &[Schema]) -> String {
        let serialized = serde_json::to_string(schemas).unwrap();
        debug!("Serialized schemas: {}", serialized);
        let mut hasher = Xxh3::new();
        hasher.write(serialized.as_bytes());
        format!("{:016x}", hasher.finish())
    }
}

/// Represents the C++ base namespace for the Craby project.
#[derive(Debug)]
pub struct CxxNamespace(pub String);

impl<T> From<T> for CxxNamespace
where
    T: AsRef<str>,
{
    fn from(value: T) -> Self {
        CxxNamespace(format!("craby::{}", flat_case(value.as_ref())))
    }
}

impl Display for CxxNamespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents the C++ TurboModule class name. (eg. `CxxFastCalculatorModule`)
#[derive(Debug)]
pub struct CxxModuleName(pub String);

impl<T> From<T> for CxxModuleName
where
    T: AsRef<str>,
{
    fn from(value: T) -> Self {
        CxxModuleName(format!("Cxx{}Module", pascal_case(value.as_ref())))
    }
}

impl Display for CxxModuleName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Represents the Objective-C module provider name. (eg. `FastCalculatorModuleProvider`)
#[derive(Debug)]
pub struct ObjCProviderName(pub String);

impl<T> From<T> for ObjCProviderName
where
    T: AsRef<str>,
{
    fn from(value: T) -> Self {
        ObjCProviderName(format!("{}ModuleProvider", pascal_case(value.as_ref())))
    }
}

impl Display for ObjCProviderName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
