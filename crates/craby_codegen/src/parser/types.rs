use std::hash::{DefaultHasher, Hash, Hasher};

use oxc::{diagnostics::OxcDiagnostic, semantic::ReferenceId};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("General error")]
    General(#[from] anyhow::Error),
    #[error("Oxc error")]
    Oxc { diagnostics: Vec<OxcDiagnostic> },
}

#[derive(Debug)]
pub struct Spec {
    /// Spec name
    pub name: String,
    /// Module methods
    pub methods: Vec<Method>,
    /// Module signals
    pub signals: Vec<Signal>,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct Method {
    pub name: String,
    pub params: Vec<Param>,
    pub ret_type: TypeAnnotation,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct Param {
    pub name: String,
    pub type_annotation: TypeAnnotation,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Hash)]
pub enum TypeAnnotation {
    Void,
    Boolean,
    Number,
    String,
    Array(Box<TypeAnnotation>),
    ArrayBuffer,
    Object(ObjectTypeAnnotation),
    Enum(EnumTypeAnnotation),
    Promise(Box<TypeAnnotation>),
    Nullable(Box<TypeAnnotation>),
    // Reference to `TypeAnnotation::Object` or `TypeAnnotation::Enum` or Alias types (eg. `Promise`)
    Ref(RefTypeAnnotation),
}

impl TypeAnnotation {
    pub fn to_id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    pub fn as_object(&self) -> Option<&ObjectTypeAnnotation> {
        match self {
            TypeAnnotation::Object(object) => Some(object),
            _ => None,
        }
    }

    pub fn as_enum(&self) -> Option<&EnumTypeAnnotation> {
        match self {
            TypeAnnotation::Enum(enum_type) => Some(enum_type),
            _ => None,
        }
    }

    pub fn is_nullable(&self) -> bool {
        matches!(self, TypeAnnotation::Nullable(..))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Hash)]
pub struct ObjectTypeAnnotation {
    pub name: String,
    pub props: Vec<Prop>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Hash)]
pub struct Prop {
    pub name: String,
    pub type_annotation: TypeAnnotation,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Hash)]
pub struct EnumTypeAnnotation {
    pub name: String,
    pub members: Vec<EnumMember>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Hash)]
pub struct EnumMember {
    pub name: String,
    pub value: EnumMemberValue,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Hash)]
pub enum EnumMemberValue {
    String(String),
    Number(usize),
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Hash)]
pub struct RefTypeAnnotation {
    #[serde(skip)]
    pub ref_id: ReferenceId,
    pub name: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct Signal {
    pub name: String,
    pub payload_type: Option<TypeAnnotation>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_id() {
        let t1 = TypeAnnotation::Object(ObjectTypeAnnotation {
            name: "Object".to_string(),
            props: vec![Prop {
                name: "prop".to_string(),
                type_annotation: TypeAnnotation::String,
            }],
        });

        let t2 = TypeAnnotation::Object(ObjectTypeAnnotation {
            name: "Object".to_string(),
            props: vec![Prop {
                name: "prop".to_string(),
                type_annotation: TypeAnnotation::String,
            }],
        });

        let t3 = TypeAnnotation::Object(ObjectTypeAnnotation {
            name: "Object".to_string(),
            props: vec![
                Prop {
                    name: "prop".to_string(),
                    type_annotation: TypeAnnotation::String,
                },
                Prop {
                    name: "prop2".to_string(),
                    type_annotation: TypeAnnotation::String,
                },
            ],
        });

        assert_eq!(t1.to_id(), t2.to_id());
        assert_ne!(t1.to_id(), t3.to_id());
    }
}
