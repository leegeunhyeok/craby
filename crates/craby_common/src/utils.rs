use convert_case::{Case, Casing};
use regex::Regex;

pub fn sanitize_str(value: &str) -> String {
    let re = Regex::new(r"[^a-zA-Z]").unwrap();
    re.replace_all(&value, "_").to_case(Case::Snake).to_string()
}
