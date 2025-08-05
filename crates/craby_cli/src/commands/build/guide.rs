use owo_colors::OwoColorize;

use crate::utils::terminal::CodeHighlighter;

pub fn print_guide(lib_name: &String) {
    print_usage(lib_name);
}

fn print_usage(lib_name: &String) {
    let highlighter = CodeHighlighter::new();

    println!("\nAndroid setup and usage:\n");
    println!(
        "Open `{}` file and add the following line:\n",
        "android/build.gradle".underline()
    );
    let content = r#"android {
  // ...

  sourceSets {
    main {
      // Add this line
      jniLibs.srcDirs += ["src/main/jniLibs"]
    }
  }
}"#;
    println!("```gradle");
    highlighter.highlight_code(&content.to_string(), "gradle");
    println!();
    println!("```\n");

    let content = format!(
        r#"@ReactModule(name = SomeModule.NAME)
class SomeModule(reactContext: ReactApplicationContext) :
  NativeSomeModuleSpec(reactContext) {{
  
  init {{
    // Load static library to use native methods
    System.loadLibrary("{}")
  }}

  // Declare the native method
  private external fun nativeSomeMethod(a: Double, b: Double): Double

  // ...

  override fun someMethod(a: Double, b: Double): Double {{
    // Call the native method
    return nativeSomeMethod(a, b);
  }}
}}"#,
        lib_name
    );
    println!("```kt");
    highlighter.highlight_code(&content, "java");
    println!();
    println!("```\n");

    println!("iOS setup and usage:\n");
    println!(
        "Open `{}` file and add the following line:\n",
        "<ModuleName>.podspec".underline()
    );
    let content = format!(
        r#"Pod::Spec.new do |s|
  # ...

  # Add this line to use Rust module
  s.vendored_frameworks = "ios/framework/lib{}.xcframework"
end"#,
        lib_name
    );
    println!("```rb");
    highlighter.highlight_code(&content, "rb");
    println!();
    println!("```\n");

    let content = format!(
        r#"import "SomeModule.h"
import "lib{}.h" // Add this line

@implementation SomeModule
RCT_EXPORT_MODULE()

- (NSNumber *)someMethod:(double *)a b:(double *)b {{
  // Call the native method
  NSNumber *result = @(someMethod(a, b));
  return result;
}}

// ...

@end"#,
        lib_name
    );
    println!("```objc");
    highlighter.highlight_code(&content, "mm");
    println!();
    println!("```\n");
}
