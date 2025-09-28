use oxc::{diagnostics::OxcDiagnostic, semantic::ReferenceId};
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
    /// Module methods
    pub methods: Vec<Method>,
    /// Module signals
    pub signals: Vec<Signal>,
}

#[derive(Debug)]
pub struct Method {
    pub name: String,
    pub params: Vec<Param>,
    pub ret_type: TypeAnnotation,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Param {
    pub name: String,
    pub type_annotation: TypeAnnotation,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum TypeAnnotation {
    Void,
    Boolean,
    Number,
    String,
    Array(Box<TypeAnnotation>),
    Object(ObjectTypeAnnotation),
    Enum(EnumTypeAnnotation),
    Promise(Box<TypeAnnotation>),
    Nullable(Box<TypeAnnotation>),
    // Reference to `TypeAnnotation::Object` or `TypeAnnotation::Enum` or Alias types (eg. `Promise`)
    Ref(RefTypeAnnotation),
}

impl TypeAnnotation {
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
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ObjectTypeAnnotation {
    pub name: String,
    pub props: Vec<Prop>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Prop {
    pub name: String,
    pub type_annotation: TypeAnnotation,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct EnumTypeAnnotation {
    pub name: String,
    pub members: Vec<EnumMember>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct EnumMember {
    pub name: String,
    pub value: EnumMemberValue,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum EnumMemberValue {
    String(String),
    Number(usize),
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RefTypeAnnotation {
    pub ref_id: ReferenceId,
    pub name: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Signal {
    pub name: String,
}
