use craby_common::utils::{
    ios::xcframework_name,
    string::{kebab_case, SanitizedString},
};
use indoc::formatdoc;
use owo_colors::OwoColorize;

use crate::utils::terminal::CodeHighlighter;

pub fn print_guide(mod_name: &String) {
    let sanitized_mod_name = SanitizedString::from(mod_name);
    let highlighter = CodeHighlighter::new();

    // Android
    println!(
        "{}",
        formatdoc! {
            r#"

            👉 Android setup:

            Open `{gradle_path}` file and add the following line:
            "#,
            gradle_path = "android/build.gradle".underline(),
        }
    );
    let gradle_code = formatdoc! {
        r#"
        android {{
          externalNativeBuild {{
            // Add CMake build configuration
            cmake {{
              path "CMakeLists.txt"
              targets "cxx-{kebab_name}"

              // ...
            }}
          }}
        }}"#,
        kebab_name = kebab_case(mod_name),
    };
    highlighter.highlight_code_with_box(&gradle_code, "gradle");

    // iOS
    println!(
        "{}",
        formatdoc! {
            r#"
        
            👉 iOS setup:
            
            Open `{podspec_path}` file and add the following line:
            "#,
            podspec_path = "<ModuleName>.podspec".underline(),
        }
    );
    let podspec_content = formatdoc! {
        r#"
        Pod::Spec.new do |s|
          # Add these lines
          s.source_files = ["ios/**/*.{{h,m,mm,cc,cpp}}", "cpp/**/*.{{hpp,cpp}}"]
          s.private_header_files = "ios/include/*.h"
          s.vendored_frameworks = "ios/framework/{xcframework_name}"
        end"#,
        xcframework_name = xcframework_name(&sanitized_mod_name),
    };
    highlighter.highlight_code_with_box(&podspec_content, "rb");
}
