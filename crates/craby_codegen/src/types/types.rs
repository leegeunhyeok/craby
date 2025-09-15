use super::schema::TypeAnnotation;

pub trait ToType {
    fn to_type(&self) -> Result<Type, anyhow::Error>;
}

pub enum Type {
    String,
    Number,
    Boolean,
    Void,
    Array(String),
    Nullable(String),
    Object,
    Alias(String),
    Enum(String),
    Promise(String),
}

impl ToString for Type {
    fn to_string(&self) -> String {
        match self {
            Type::String => "String".to_string(),
            Type::Number => "f64".to_string(),
            Type::Boolean => "bool".to_string(),
            Type::Void => "()".to_string(),
            Type::Array(t) => format!("Vec<{}>", t),
            Type::Nullable(t) => format!("Option<{}>", t),
            Type::Object => "()".to_string(),
            Type::Alias(t) => t.clone(),
            Type::Enum(name) => name.clone(),
            Type::Promise(t) => format!("Result<{}, anyhow::Error>", t),
        }
    }
}

impl ToType for TypeAnnotation {
    fn to_type(&self) -> Result<Type, anyhow::Error> {
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

            // Enum
            TypeAnnotation::EnumDeclaration {
                name, member_type, ..
            } => match member_type.as_str() {
                "NumberTypeAnnotation" => Type::Enum(name.clone()),
                "StringTypeAnnotation" => Type::Enum(name.clone()),
                _ => return Err(anyhow::anyhow!("Unsupported enum type: {}", member_type)),
            },

            // Union type
            TypeAnnotation::UnionTypeAnnotation { member_type } => match member_type.as_str() {
                "NumberTypeAnnotation" => Type::Number,
                "StringTypeAnnotation" => Type::String,
                _ => return Err(anyhow::anyhow!("Unsupported union type: {}", member_type)),
            },

            // Promise type
            TypeAnnotation::PromiseTypeAnnotation { element_type } => {
                Type::Promise(element_type.to_type()?.to_string())
            }

            // Void type
            TypeAnnotation::VoidTypeAnnotation => Type::Void,

            // Unsupported types
            TypeAnnotation::FunctionTypeAnnotation { .. }
            | TypeAnnotation::ObjectTypeAnnotation { .. } => {
                return Err(anyhow::anyhow!("Unsupported type annotation: {:?}", self));
            }

            _ => {
                return Err(anyhow::anyhow!("Unsupported type annotation: {:?}", self));
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
}
