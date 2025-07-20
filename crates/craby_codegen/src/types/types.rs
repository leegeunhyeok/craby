pub enum Type {
    String,
    Number,
    Boolean,
    Void,
    Array(String),
    Nullable(String)
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
        }
    }
}
