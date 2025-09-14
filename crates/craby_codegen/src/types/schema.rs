use std::collections::HashMap;

use craby_common::utils::string::{pascal_case, snake_case};
use indoc::formatdoc;
use log::error;
use serde::{Deserialize, Serialize};

use super::types::Type;

#[derive(Debug, Deserialize, Serialize)]
pub struct Schema {
    #[serde(rename = "moduleName")]
    pub module_name: String,
    // NativeModule, Component
    pub r#type: String,
    #[serde(rename = "aliasMap")]
    pub alias_map: HashMap<String, Alias>,
    #[serde(rename = "enumMap")]
    pub enum_map: HashMap<String, String>,
    pub spec: Spec,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Alias {
    pub r#type: String,
    pub properties: Vec<AliasProperty>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AliasProperty {
    pub name: String,
    pub optional: bool,
    #[serde(rename = "typeAnnotation")]
    pub type_annotation: TypeAnnotation,
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
    pub fn to_type(&self) -> Result<Type, anyhow::Error> {
        let r#type = match self {
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

            // Array type
            TypeAnnotation::ArrayTypeAnnotation { element_type } => {
                Type::Array(element_type.to_type()?.to_string())
            }

            // Type alias
            TypeAnnotation::TypeAliasTypeAnnotation { name } => Type::Alias(name.clone()),

            // Void type
            TypeAnnotation::VoidTypeAnnotation => Type::Void,

            // Unsupported types
            TypeAnnotation::FunctionTypeAnnotation { .. }
            | TypeAnnotation::ObjectTypeAnnotation { .. } => {
                return Err(anyhow::anyhow!("Unsupported type annotation: {:?}", self));
            }

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
                //         Type::Array(element_type.to_type())
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
                //         Type::Nullable(type_annotation.to_type())
                //     }

                //     // Type alias
                //     TypeAnnotation::TypeAliasTypeAnnotation { .. } => {
                //         unimplemented!("TypeAliasTypeAnnotation")
                //     }
                // }
            }
        };

        Ok(r#type)
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
    pub fn to_sig(&self) -> Result<String, anyhow::Error> {
        if let TypeAnnotation::ObjectTypeAnnotation { .. }
        | TypeAnnotation::GenericObjectTypeAnnotation { .. } = self.type_annotation
        {
            error!("Object type is not supported for parameters");
            error!("Use defined type alias instead");
            unimplemented!();
        }

        if let TypeAnnotation::FunctionTypeAnnotation { .. } = self.type_annotation {
            error!("Function type is not supported for parameters");
            unimplemented!();
        }

        let (type_annotation, is_nullable) = self.type_annotation.unwrap_nullable();
        let param_type = type_annotation.to_type()?.to_string();

        let final_type = if self.optional && !is_nullable {
            format!("Option<{}>", param_type)
        } else if is_nullable || self.optional {
            if param_type.starts_with("Option<") {
                param_type
            } else {
                format!("Option<{}>", param_type)
            }
        } else {
            param_type
        };

        Ok(format!("{}: {}", self.name, final_type))
    }
}

impl FunctionSpec {
    pub fn to_sig(&self) -> Result<String, anyhow::Error> {
        match &self.type_annotation {
            TypeAnnotation::FunctionTypeAnnotation {
                return_type_annotation,
                params,
            } => {
                let return_type = return_type_annotation.to_type()?.to_string();
                let params_sig = params
                    .iter()
                    .map(|p| p.to_sig())
                    .collect::<Result<Vec<_>, _>>()
                    .map(|p| p.join(", "))?;

                let fn_name = snake_case(&self.name);
                let ret_annotation = if return_type == "()" {
                    String::new()
                } else {
                    format!(" -> {}", return_type)
                };

                Ok(format!(
                    "fn {}({}){}",
                    fn_name.to_string(),
                    params_sig,
                    ret_annotation
                ))
            }
            _ => unimplemented!("Unsupported type annotation for function: {}", self.name),
        }
    }

    /// Returns the CXX(FFI) function signature for the `FunctionSpec`.
    ///
    /// ```rust,ignore
    /// // extern function
    /// #[cxx_name = "myFunc"]
    /// fn myFunc(arg1: Foo, arg2: Bar) -> Baz;
    ///
    /// // impl function
    /// fn myFunc(arg1: Foo, arg2: Bar) -> Baz {
    ///     MyModule::my_func(arg1, arg2)
    /// }
    /// ```
    pub fn to_cxx_func(&self, mod_name: &String) -> Result<CxxFunction, anyhow::Error> {
        match &self.type_annotation {
            TypeAnnotation::FunctionTypeAnnotation {
                return_type_annotation,
                params,
            } => {
                let return_type = return_type_annotation.to_type()?.to_string();
                let params_sig = params
                    .iter()
                    .map(|p| p.to_sig())
                    .collect::<Result<Vec<_>, _>>()
                    .map(|p| p.join(", "))?;

                let impl_name = pascal_case(mod_name);
                let fn_name = snake_case(&self.name);
                let fn_args = params.iter().map(|p| p.name.clone()).collect::<Vec<_>>();

                // If the return type is `void`, return an empty tuple.
                // Otherwise, return the given return type.
                let ret_annotation = if return_type == "()" {
                    String::new()
                } else {
                    format!(" -> {}", return_type)
                };

                let extern_func = formatdoc! {
                    r#"
                    #[cxx_name = "{orig_fn_name}"]
                    fn {fn_name}({params_sig}){ret};"#,
                    orig_fn_name = self.name,
                    fn_name = fn_name,
                    params_sig = params_sig,
                    ret = ret_annotation,
                };

                let impl_func = formatdoc! {
                    r#"
                    fn {fn_name}({params_sig}){ret} {{
                        {impl_name}::{fn_name}({fn_args})
                    }}"#,
                    params_sig = params_sig,
                    ret = ret_annotation,
                    impl_name = impl_name,
                    fn_name = fn_name.to_string(),
                    fn_args = fn_args.join(", "),
                };

                Ok(CxxFunction {
                    extern_func,
                    impl_func,
                })
            }
            _ => unimplemented!("Unsupported type annotation for function: {}", self.name),
        }
    }
}

pub struct CxxFunction {
    pub extern_func: String,
    pub impl_func: String,
}
