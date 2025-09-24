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
}

#[derive(Debug)]
pub struct Method {
    pub name: String,
    pub params: Vec<Param>,
    pub ret_type: TypeAnnotation,
}

#[derive(Debug, PartialEq)]
pub struct Param {
    pub name: String,
    pub type_annotation: TypeAnnotation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Prop {
    pub name: String,
    pub type_annotation: TypeAnnotation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumMember {
    pub name: String,
    pub value: EnumMemberValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnumMemberValue {
    String(String),
    Number(f64),
}

#[derive(Debug, Clone, PartialEq)]
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
    // Reference to `TypeAnnotation::Object` or `TypeAnnotation::Enum`
    Ref(ReferenceId),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectTypeAnnotation {
    pub name: String,
    pub props: Vec<Prop>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumTypeAnnotation {
    pub name: String,
    pub members: Vec<EnumMember>,
}
