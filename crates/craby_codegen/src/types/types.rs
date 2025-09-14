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
