use std::collections::HashMap;

use craby_common::constants;
use serde::{Deserialize, Serialize};

use crate::utils::to_jni_fn_name;

use super::types::Type;

#[derive(Debug, Deserialize, Serialize)]
pub struct Schema {
    #[serde(rename = "moduleName")]
    pub module_name: String,
    pub r#type: String,
    #[serde(rename = "aliasMap")]
    pub alias_map: HashMap<String, String>,
    #[serde(rename = "enumMap")]
    pub enum_map: HashMap<String, String>,
    pub spec: Spec,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Spec {
    #[serde(rename = "eventEmitters")]
    pub event_emitters: Vec<String>,
    pub methods: Vec<FunctionSpec>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum TypeAnnotation {
    // Reserved types
    ReservedTypeAnnotation {
        name: String,
    },

    // String types
    StringTypeAnnotation,
    StringLiteralTypeAnnotation {
        value: String,
    },
    StringLiteralUnionTypeAnnotation {
        values: Vec<String>,
    },

    // Boolean type
    BooleanTypeAnnotation,

    // Number types
    NumberTypeAnnotation,
    FloatTypeAnnotation,
    DoubleTypeAnnotation,
    Int32TypeAnnotation,
    NumberLiteralTypeAnnotation {
        value: f64,
    },

    // Enum
    EnumDeclaration {
        #[serde(rename = "memberType")]
        member_type: String,
        members: Vec<EnumMember>,
    },

    // Array type
    ArrayTypeAnnotation {
        #[serde(rename = "elementType")]
        element_type: Box<TypeAnnotation>,
    },

    // Function type
    #[serde(rename = "FunctionTypeAnnotation")]
    FunctionTypeAnnotation {
        #[serde(rename = "returnTypeAnnotation")]
        return_type_annotation: Box<TypeAnnotation>,
        params: Vec<Parameter>,
    },

    // Object types
    GenericObjectTypeAnnotation,
    ObjectTypeAnnotation {
        properties: Option<Vec<ObjectProperty>>,
    },

    // Union type
    UnionTypeAnnotation {
        #[serde(rename = "memberType")]
        member_type: String,
        types: Vec<TypeAnnotation>,
    },

    // Mixed type
    MixedTypeAnnotation,

    // Void type
    VoidTypeAnnotation,

    // Nullable wrapper
    NullableTypeAnnotation {
        #[serde(rename = "typeAnnotation")]
        type_annotation: Box<TypeAnnotation>,
    },

    // Type alias
    TypeAliasTypeAnnotation {
        name: String,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EnumMember {
    pub name: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ObjectProperty {
    pub name: String,
    pub optional: bool,
    #[serde(rename = "typeAnnotation")]
    pub type_annotation: TypeAnnotation,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Parameter {
    pub name: String,
    pub optional: bool,
    #[serde(rename = "typeAnnotation")]
    pub type_annotation: TypeAnnotation,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FunctionSpec {
    pub name: String,
    pub optional: bool,
    #[serde(rename = "typeAnnotation")]
    pub type_annotation: TypeAnnotation,
}

impl TypeAnnotation {
    pub fn to_rs_type(&self) -> String {
        match self {
            // Reserved types
            TypeAnnotation::ReservedTypeAnnotation { name } => match name.as_str() {
                "RootTag" => Type::Number,
                _ => unimplemented!("Unknown reserved type: {}", name),
            },

            // String types
            TypeAnnotation::StringTypeAnnotation => Type::String,
            TypeAnnotation::StringLiteralTypeAnnotation { .. } => Type::String,
            TypeAnnotation::StringLiteralUnionTypeAnnotation { .. } => Type::String,

            // Boolean type
            TypeAnnotation::BooleanTypeAnnotation => Type::Boolean,

            // Number types
            TypeAnnotation::NumberTypeAnnotation => Type::Number,
            TypeAnnotation::FloatTypeAnnotation => Type::Number,
            TypeAnnotation::DoubleTypeAnnotation => Type::Number,
            TypeAnnotation::Int32TypeAnnotation => Type::Number,
            TypeAnnotation::NumberLiteralTypeAnnotation { .. } => Type::Number,

            // Enum
            TypeAnnotation::EnumDeclaration { member_type, .. } => match member_type.as_str() {
                "NumberTypeAnnotation" => Type::Number,
                "StringTypeAnnotation" => Type::String,
                _ => unimplemented!("Unknown enum type: {}", member_type),
            },

            // Array type
            TypeAnnotation::ArrayTypeAnnotation { element_type } => {
                Type::Array(element_type.to_rs_type())
            }

            // Function type
            TypeAnnotation::FunctionTypeAnnotation { .. } => {
                unimplemented!("FunctionTypeAnnotation")
            }

            // Object types
            TypeAnnotation::GenericObjectTypeAnnotation => {
                unimplemented!("GenericObjectTypeAnnotation");
            }
            TypeAnnotation::ObjectTypeAnnotation { .. } => {
                unimplemented!("ObjectTypeAnnotation");
            }

            // Union type
            TypeAnnotation::UnionTypeAnnotation { member_type, .. } => match member_type.as_str() {
                // TODO: Enum type support
                "NumberTypeAnnotation" => Type::Number,
                "StringTypeAnnotation" => Type::String,
                "ObjectTypeAnnotation" => unimplemented!("ObjectTypeAnnotation"),
                _ => unimplemented!("Unknown union type: {}", member_type),
            },

            // Mixed type
            TypeAnnotation::MixedTypeAnnotation => unimplemented!("MixedTypeAnnotation"),

            // Void type
            TypeAnnotation::VoidTypeAnnotation => Type::Void,

            // Nullable wrapper
            TypeAnnotation::NullableTypeAnnotation { type_annotation } => {
                Type::Nullable(type_annotation.to_rs_type())
            }

            // Type alias
            TypeAnnotation::TypeAliasTypeAnnotation { .. } => {
                unimplemented!("TypeAliasTypeAnnotation")
            }
        }
        .to_string()
    }

    /// Unwrap nullable type annotations to get the inner type and nullable flag
    pub fn unwrap_nullable(&self) -> (&TypeAnnotation, bool) {
        match self {
            TypeAnnotation::NullableTypeAnnotation { type_annotation } => {
                let (inner, _) = type_annotation.unwrap_nullable();
                (inner, true)
            }
            _ => (self, false),
        }
    }
}

impl Parameter {
    pub fn to_rs_param(&self) -> String {
        let (type_annotation, is_nullable) = self.type_annotation.unwrap_nullable();
        let rust_type = type_annotation.to_rs_type();

        let final_type = if self.optional && !is_nullable {
            format!("Option<{}>", rust_type)
        } else if is_nullable || self.optional {
            if rust_type.starts_with("Option<") {
                rust_type
            } else {
                format!("Option<{}>", rust_type)
            }
        } else {
            rust_type
        };

        format!("{}: {}", self.name, final_type)
    }
}

impl FunctionSpec {
    pub fn to_rs_fn(&self, ident: usize) -> String {
        match &self.type_annotation {
            TypeAnnotation::FunctionTypeAnnotation {
                return_type_annotation,
                params,
            } => {
                let return_type = return_type_annotation.to_rs_type();
                let params_sig = params
                    .iter()
                    .map(|p| p.to_rs_param())
                    .collect::<Vec<_>>()
                    .join(", ");
                let params = params
                    .iter()
                    .map(|p| p.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ");

                let ret_annotation = if return_type == "()" {
                    String::new()
                } else {
                    format!(" -> {}", return_type)
                };

                format!(
                    "{ident}pub fn {name}({params_sig}){ret_annotation} {{\n    {ident}{body}\n{ident}}}",
                    name = self.name,
                    params_sig = params_sig,
                    ret_annotation = ret_annotation,
                    body = format!("{}::{}({})", constants::IMPL_MOD_NAME, self.name, params),
                    ident = " ".repeat(ident)
                )
            }
            _ => unimplemented!("Unsupported type annotation for function: {}", self.name),
        }
    }

    pub fn to_android_ffi_fn(
        &self,
        lib_name: &String,
        mod_name: &String,
        java_package_name: &String,
        class_name: &String,
    ) -> String {
        match &self.type_annotation {
            TypeAnnotation::FunctionTypeAnnotation {
                return_type_annotation,
                params,
            } => {
                let jni_fn_name = to_jni_fn_name(&self.name, java_package_name, class_name);
                let return_type = return_type_annotation.to_rs_type();
                let params_sig = params
                    .iter()
                    .map(|p| p.to_rs_param())
                    .collect::<Vec<_>>()
                    .join(", ");
                let params_sig = [
                    "_env: JNIEnv".to_string(),
                    "_class: jobject".to_string(),
                    params_sig,
                ]
                .join(", ");
                let params = params
                    .iter()
                    .map(|p| p.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ");

                let ret_annotation = if return_type == "()" {
                    String::new()
                } else {
                    format!(" -> {}", return_type)
                };

                format!(
                    "#[no_mangle]\npub extern \"C\" fn {name}({params_sig}){ret_annotation} {{\n    {body}\n}}",
                    name = jni_fn_name,
                    params_sig = params_sig,
                    ret_annotation = ret_annotation,
                    body = format!("{}::{}::{}({})", lib_name,mod_name, self.name, params),
                )
            }
            _ => unimplemented!("Unsupported type annotation for function: {}", self.name),
        }
    }

    pub fn to_ios_ffi_fn(&self, lib_name: &String, mod_name: &String) -> String {
        let code = self.to_rs_fn(0);
        let code = code.replace(
            format!("pub fn {}", self.name).as_str(),
            format!("pub extern \"C\" fn {}", self.name).as_str(),
        );
        let code = code.replace(
            format!("{}::{}", constants::IMPL_MOD_NAME, self.name).as_str(),
            format!("{}::{}::{}", lib_name.as_str(), mod_name, self.name).as_str(),
        );

        ["#[no_mangle]".to_string(), code].join("\n")
    }
}
