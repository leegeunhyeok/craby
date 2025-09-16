use std::collections::BTreeMap;

use craby_common::utils::string::snake_case;
use log::error;
use serde::{Deserialize, Serialize};

use crate::platform::rust::ToRsType;

#[derive(Debug, Deserialize, Serialize)]
pub struct Schema {
    #[serde(rename = "moduleName")]
    pub module_name: String,
    // NativeModule, Component
    pub r#type: String,
    #[serde(rename = "aliasMap")]
    pub alias_map: BTreeMap<String, Alias>,
    #[serde(rename = "enumMap")]
    pub enum_map: BTreeMap<String, Enum>,
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
    pub type_annotation: Box<TypeAnnotation>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Enum {
    pub name: String,
    pub r#type: String,
    #[serde(rename = "memberType")]
    pub member_type: String,
    #[serde(default)]
    pub members: Option<Vec<EnumMember>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EnumMember {
    pub name: String,
    pub value: EnumMemberValue,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EnumMemberValue {
    pub r#type: String,
    pub value: String,
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

    // String types
    StringTypeAnnotation,
    StringLiteralTypeAnnotation {
        value: String,
    },
    StringLiteralUnionTypeAnnotation {
        types: Vec<Box<TypeAnnotation>>,
    },

    // Array type
    ArrayTypeAnnotation {
        #[serde(rename = "elementType")]
        element_type: Box<TypeAnnotation>,
    },

    // Enum
    EnumDeclaration {
        name: String,
        #[serde(rename = "memberType")]
        member_type: String,
        #[serde(default)]
        members: Option<Vec<EnumMember>>,
    },

    // Object types
    GenericObjectTypeAnnotation,
    ObjectTypeAnnotation {
        properties: Option<Vec<ObjectProperty>>,
    },

    // Function type
    FunctionTypeAnnotation {
        #[serde(rename = "returnTypeAnnotation")]
        return_type_annotation: Box<TypeAnnotation>,
        params: Vec<Parameter>,
    },

    // Union type
    UnionTypeAnnotation {
        #[serde(rename = "memberType")]
        member_type: String,
    },

    // Promise type
    PromiseTypeAnnotation {
        #[serde(rename = "elementType")]
        element_type: Box<TypeAnnotation>,
    },

    // Mixed type
    MixedTypeAnnotation,

    // Void type
    VoidTypeAnnotation,

    // Nullable type
    NullableTypeAnnotation {
        #[serde(rename = "typeAnnotation")]
        type_annotation: Box<TypeAnnotation>,
    },

    // Type alias
    TypeAliasTypeAnnotation {
        name: String,
    },
}

impl TypeAnnotation {
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

#[derive(Debug, Deserialize, Serialize)]
pub struct ObjectProperty {
    pub name: String,
    pub optional: bool,
    #[serde(rename = "typeAnnotation")]
    pub type_annotation: Box<TypeAnnotation>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Parameter {
    pub name: String,
    pub optional: bool,
    #[serde(rename = "typeAnnotation")]
    pub type_annotation: Box<TypeAnnotation>,
}

impl Parameter {
    pub fn as_sig(&self) -> Result<String, anyhow::Error> {
        if let TypeAnnotation::ObjectTypeAnnotation { .. }
        | TypeAnnotation::GenericObjectTypeAnnotation { .. } = *self.type_annotation
        {
            error!("Object type is not supported for parameters");
            error!("Use defined type alias instead");
            unimplemented!();
        }

        if let TypeAnnotation::FunctionTypeAnnotation { .. } = *self.type_annotation {
            error!("Function type is not supported for parameters");
            unimplemented!();
        }

        let (type_annotation, is_nullable) = self.type_annotation.unwrap_nullable();
        let param_type = type_annotation.to_rs_type()?;

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

#[derive(Debug, Deserialize, Serialize)]
pub struct FunctionSpec {
    pub name: String,
    pub optional: bool,
    #[serde(rename = "typeAnnotation")]
    pub type_annotation: Box<TypeAnnotation>,
}

impl FunctionSpec {
    pub fn args_count(&self) -> Result<usize, anyhow::Error> {
        if let TypeAnnotation::FunctionTypeAnnotation { params, .. } = self.type_annotation.as_ref()
        {
            Ok(params.len())
        } else {
            return Err(anyhow::anyhow!(
                "Function type annotation should be a function: {}",
                self.name
            ));
        }
    }

    pub fn as_sig(&self) -> Result<String, anyhow::Error> {
        match &*self.type_annotation {
            TypeAnnotation::FunctionTypeAnnotation {
                return_type_annotation,
                params,
            } => {
                let return_type = return_type_annotation.to_rs_type()?;
                let params_sig = params
                    .iter()
                    .map(|param| param.as_sig())
                    .collect::<Result<Vec<_>, _>>()
                    .map(|params| params.join(", "))?;

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
}
