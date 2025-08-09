use craby_common::env::Platform;

pub enum Type {
    String,
    Number,
    Boolean,
    Void,
    Array(String),
    Nullable(String),
}

#[derive(Debug, Clone)]
pub struct InteropInfo {
    pub import_module: Option<String>,
    pub from_ffi_fn: Option<String>,
    pub to_ffi_fn: Option<String>,
}

impl InteropInfo {
    pub fn none() -> Self {
        Self {
            import_module: None,
            from_ffi_fn: None,
            to_ffi_fn: None,
        }
    }

    pub fn string(platform: Platform) -> Self {
        let platform = match platform {
            Platform::Android => "android",
            Platform::Ios => "ios",
        };

        Self {
            import_module: Some(format!("craby_core::{platform}::interop::string::*")),
            from_ffi_fn: Some("String::from_native".to_string()),
            to_ffi_fn: Some("to_native".to_string()),
        }
    }
}

impl Type {
    pub fn get_interop_info(&self, platform: Platform) -> InteropInfo {
        match self {
            Type::String => InteropInfo::string(platform),
            Type::Number | Type::Boolean | Type::Void => InteropInfo::none(),
            Type::Array(_) | Type::Nullable(_) => InteropInfo::none(), // TODO: Implement for complex types
        }
    }
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
