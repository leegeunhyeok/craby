pub fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

pub fn indent_str(str: String, indent_size: usize) -> String {
    let indent_str = " ".repeat(indent_size);
    str.lines()
        .map(|line| {
            if line.trim().is_empty() {
                line.to_string()
            } else {
                format!("{}{}", indent_str, line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capitalize_first() {
        assert_eq!(capitalize_first("sum"), "Sum");
        assert_eq!(capitalize_first("hello-world"), "Hello-world");
    }

    #[test]
    fn test_indent_str() {
        assert_eq!(
            indent_str("Hello\nWorld".to_string(), 2),
            "  Hello\n  World"
        );
        assert_eq!(
            indent_str("Hello\nWorld".to_string(), 4),
            "    Hello\n    World"
        );
    }
}
