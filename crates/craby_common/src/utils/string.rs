use convert_case::{Case, Casing};

#[derive(Debug, Clone)]
pub struct SanitizedString(pub String);
impl SanitizedString {
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }

    pub fn to_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for SanitizedString {
    fn from(value: &str) -> Self {
        SanitizedString(value.to_string())
    }
}

impl From<&String> for SanitizedString {
    fn from(value: &String) -> Self {
        SanitizedString(value.clone())
    }
}

pub fn pascal_case(value: &str) -> String {
    value.to_case(Case::Pascal)
}

pub fn snake_case(value: &str) -> String {
    value.to_case(Case::Snake)
}

pub fn kebab_case(value: &str) -> String {
    value.to_case(Case::Kebab)
}

pub fn flat_case(value: &str) -> String {
    value.to_case(Case::Flat)
}
