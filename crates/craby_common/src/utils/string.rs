use std::fmt::{Display, Formatter};

use convert_case::{Case, Casing};
use regex::Regex;

#[derive(Debug, Clone)]
pub struct SanitizedString(pub String);
impl SanitizedString {
    fn regex() -> Regex {
        Regex::new(r"[^a-zA-Z0-9]").unwrap()
    }

    pub fn to_str(&self) -> &str {
        &self.0
    }
}

impl Display for SanitizedString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for SanitizedString {
    fn from(value: &str) -> Self {
        SanitizedString(
            SanitizedString::regex()
                .replace_all(value.to_lowercase().as_str(), "_")
                .to_string(),
        )
    }
}

impl From<&String> for SanitizedString {
    fn from(value: &String) -> Self {
        SanitizedString(
            SanitizedString::regex()
                .replace_all(value.to_lowercase().as_str(), "_")
                .to_string(),
        )
    }
}

pub fn pascal_case(value: &str) -> String {
    value.to_case(Case::Pascal)
}

pub fn camel_case(value: &str) -> String {
    value.to_case(Case::Camel)
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
