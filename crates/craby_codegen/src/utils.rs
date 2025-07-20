pub fn to_jni_fn_name(fn_name: &String, java_package_name: &String, class_name: &String) -> String {
    // fn_name: sum,
    // class_name: MathModule
    // java_package_name: com.example.app
    // = "Java_com_example_app_MathModule_nativeSum"
    [
        "Java".to_string(),
        java_package_name.replace(".", "_"),
        class_name.clone(),
        format!("native{}", capitalize_first(fn_name)),
    ]
    .join("_")
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_jni_fn_name() {
        assert_eq!(
            to_jni_fn_name(
                &"sum".to_string(),
                &"com.example.app".to_string(),
                &"MathModule".to_string()
            ),
            "Java_com_example_app_MathModule_nativeSum"
        );
    }

    #[test]
    fn test_capitalize_first() {
        assert_eq!(capitalize_first("sum"), "Sum");
        assert_eq!(capitalize_first("hello-world"), "Hello-world");
    }
}
