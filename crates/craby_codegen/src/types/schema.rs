use std::collections::HashMap;

use craby_common::{constants, env::Platform, utils::sanitize_str};
use log::error;
use serde::{Deserialize, Serialize};

use crate::utils::to_jni_fn_name;

use super::types::Type;

#[derive(Debug, Deserialize, Serialize)]
pub struct SchemaInfo {
    pub library: Library,
    #[serde(rename = "supportedApplePlatforms")]
    pub supported_apple_platforms: HashMap<String, String>,
    pub schema: SchemaMap,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SchemaMap {
    pub modules: HashMap<String, Schema>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Library {
    pub name: String,
    pub config: LibraryConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LibraryConfig {
    pub name: Option<String>,
    pub r#type: Option<String>,
    #[serde(rename = "jsSrcsDir")]
    pub js_srcs_dir: Option<String>,
    pub android: Option<AndroidConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AndroidConfig {
    #[serde(rename = "javaPackageName")]
    pub java_package_name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Schema {
    #[serde(rename = "moduleName")]
    pub module_name: String,
    // NativeModule, Component
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
            // Boolean type
            TypeAnnotation::BooleanTypeAnnotation => Type::Boolean,

            // Number types
            TypeAnnotation::NumberTypeAnnotation => Type::Number,
            TypeAnnotation::FloatTypeAnnotation => Type::Number,
            TypeAnnotation::DoubleTypeAnnotation => Type::Number,
            TypeAnnotation::Int32TypeAnnotation => Type::Number,
            TypeAnnotation::NumberLiteralTypeAnnotation { .. } => Type::Number,

            // String types
            TypeAnnotation::StringTypeAnnotation => Type::String,
            TypeAnnotation::StringLiteralTypeAnnotation { .. } => Type::String,
            TypeAnnotation::StringLiteralUnionTypeAnnotation { .. } => Type::String,

            _ => {
                error!("Unsupported type annotation: {:?}", self);
                unimplemented!();
                // match unsuported_type_annotation {
                //     // Reserved types
                //     TypeAnnotation::ReservedTypeAnnotation { name } => match name.as_str() {
                //         "RootTag" => Type::Number,
                //         _ => unimplemented!("Unknown reserved type: {}", name),
                //     },

                //     // Enum
                //     TypeAnnotation::EnumDeclaration { member_type, .. } => {
                //         match member_type.as_str() {
                //             "NumberTypeAnnotation" => Type::Number,
                //             "StringTypeAnnotation" => Type::String,
                //             _ => unimplemented!("Unknown enum type: {}", member_type),
                //         }
                //     }

                //     // Array type
                //     TypeAnnotation::ArrayTypeAnnotation { element_type } => {
                //         Type::Array(element_type.to_rs_type())
                //     }

                //     // Function type
                //     TypeAnnotation::FunctionTypeAnnotation { .. } => {
                //         unimplemented!("FunctionTypeAnnotation")
                //     }

                //     // Object types
                //     TypeAnnotation::GenericObjectTypeAnnotation => {
                //         unimplemented!("GenericObjectTypeAnnotation");
                //     }
                //     TypeAnnotation::ObjectTypeAnnotation { .. } => {
                //         unimplemented!("ObjectTypeAnnotation");
                //     }

                //     // Union type
                //     TypeAnnotation::UnionTypeAnnotation { member_type, .. } => {
                //         match member_type.as_str() {
                //             // TODO: Enum type support
                //             "NumberTypeAnnotation" => Type::Number,
                //             "StringTypeAnnotation" => Type::String,
                //             "ObjectTypeAnnotation" => unimplemented!("ObjectTypeAnnotation"),
                //             _ => unimplemented!("Unknown union type: {}", member_type),
                //         }
                //     }

                //     // Mixed type
                //     TypeAnnotation::MixedTypeAnnotation => unimplemented!("MixedTypeAnnotation"),

                //     // Void type
                //     TypeAnnotation::VoidTypeAnnotation => Type::Void,

                //     // Nullable wrapper
                //     TypeAnnotation::NullableTypeAnnotation { type_annotation } => {
                //         Type::Nullable(type_annotation.to_rs_type())
                //     }

                //     // Type alias
                //     TypeAnnotation::TypeAliasTypeAnnotation { .. } => {
                //         unimplemented!("TypeAliasTypeAnnotation")
                //     }
                // }
            }
        }
        .to_string()
    }

    pub fn to_ffi_type(&self, platform: Platform) -> String {
        let ffi_type = match platform {
            Platform::Android => match self {
                // Boolean type
                TypeAnnotation::BooleanTypeAnnotation => "bool",

                // Number types
                TypeAnnotation::NumberTypeAnnotation
                | TypeAnnotation::FloatTypeAnnotation
                | TypeAnnotation::DoubleTypeAnnotation
                | TypeAnnotation::Int32TypeAnnotation
                | TypeAnnotation::NumberLiteralTypeAnnotation { .. } => "jdouble",

                // String types
                TypeAnnotation::StringTypeAnnotation
                | TypeAnnotation::StringLiteralTypeAnnotation { .. }
                | TypeAnnotation::StringLiteralUnionTypeAnnotation { .. } => "jstring",

                _ => {
                    error!("Unsupported type annotation: {:?}", self);
                    unimplemented!();
                }
            },
            Platform::Ios => match self {
                // Boolean type
                TypeAnnotation::BooleanTypeAnnotation => "bool",

                // Number types
                TypeAnnotation::NumberTypeAnnotation
                | TypeAnnotation::FloatTypeAnnotation
                | TypeAnnotation::DoubleTypeAnnotation
                | TypeAnnotation::Int32TypeAnnotation
                | TypeAnnotation::NumberLiteralTypeAnnotation { .. } => "c_double",

                // String types
                TypeAnnotation::StringTypeAnnotation
                | TypeAnnotation::StringLiteralTypeAnnotation { .. }
                | TypeAnnotation::StringLiteralUnionTypeAnnotation { .. } => "*const c_char",

                _ => {
                    error!("Unsupported type annotation: {:?}", self);
                    unimplemented!();
                }
            },
        };

        ffi_type.to_string()
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

    pub fn to_ffi_param(&self, platform: Platform) -> String {
        // TODO: Handle nullable parameters
        let (type_annotation, _nullable) = self.type_annotation.unwrap_nullable();
        let ffi_type = type_annotation.to_ffi_type(platform);

        format!("{}: {}", self.name, ffi_type)
    }
}

impl FunctionSpec {
    pub fn to_rs_fn_sig(&self, sanitize: bool) -> String {
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
                let ret_annotation = if return_type == "()" {
                    String::new()
                } else {
                    format!(" -> {}", return_type)
                };
                format!(
                    "fn {}({}){}",
                    if sanitize {
                        sanitize_str(&self.name)
                    } else {
                        self.name.clone()
                    },
                    params_sig,
                    ret_annotation
                )
            }
            _ => unimplemented!("Unsupported type annotation for function: {}", self.name),
        }
    }

    pub fn to_rs_fn(&self, ident: usize, sanitize: bool) -> String {
        match &self.type_annotation {
            TypeAnnotation::FunctionTypeAnnotation { params, .. } => {
                let params = params
                    .iter()
                    .map(|p| p.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ");

                let fn_sig = self.to_rs_fn_sig(sanitize);

                format!(
                    "{ident}pub {fn_sig} {{\n    {ident}{body}\n{ident}}}",
                    fn_sig = fn_sig,
                    body = format!(
                        "{}::{}({})",
                        constants::IMPL_MOD_NAME,
                        sanitize_str(&self.name),
                        params
                    ),
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
                let return_type = return_type_annotation.to_ffi_type(Platform::Android);
                let params_sig = params
                    .iter()
                    .map(|p| p.to_ffi_param(Platform::Android))
                    .collect::<Vec<_>>()
                    .join(", ");
                let params_sig = [
                    "_env: JNIEnv".to_string(),
                    "_class: JObject".to_string(),
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
                    body = format!("{}::{}::{}({})", lib_name, mod_name, sanitize_str(&self.name), params),
                )
            }
            _ => unimplemented!("Unsupported type annotation for function: {}", self.name),
        }
    }

    pub fn to_ios_ffi_fn(&self, lib_name: &String, mod_name: &String) -> String {
        match &self.type_annotation {
            TypeAnnotation::FunctionTypeAnnotation {
                return_type_annotation,
                params,
            } => {
                let sanitized_name: String = sanitize_str(&self.name);
                let return_type = return_type_annotation.to_ffi_type(Platform::Ios);
                let params_sig = params
                    .iter()
                    .map(|p| p.to_ffi_param(Platform::Ios))
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
                    "#[no_mangle]\npub extern \"C\" fn {name}({params_sig}){ret_annotation} {{\n    {body}\n}}",
                    name = self.name,
                    params_sig = params_sig,
                    ret_annotation = ret_annotation,
                    body = format!("{}::{}::{}({})", lib_name, mod_name, sanitized_name, params),
                )
            }
            _ => unimplemented!("Unsupported type annotation for function: {}", self.name),
        }
    }
}
