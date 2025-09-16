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
