use convert_case::{Case, Casing};
use regex::Regex;

#[derive(Debug, Clone)]
pub struct SanitizedString(pub String);
impl SanitizedString {
    pub fn to_string(&self) -> String {
        self.0.clone()
    }

    pub fn to_str(&self) -> &str {
        &self.0
    }
}

pub fn sanitize(value: &str) -> SanitizedString {
    let re = Regex::new(r"[^a-zA-Z]").unwrap();
    let str = snake_case(re.replace_all(&value, "_").to_string().as_str());
    SanitizedString(str)
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
