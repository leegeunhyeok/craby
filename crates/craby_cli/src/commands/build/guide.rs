use owo_colors::OwoColorize;

use crate::utils::terminal::highlight_code;

pub fn print_guide(lib_name: &String) {
    print_usage(lib_name);
}

fn print_usage(lib_name: &String) {
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
    highlight_code(&content.to_string(), "gradle");
    println!();
    println!("```\n");

    let content = format!(
        r#"@ReactModule(name = MathModule.NAME)
class MathModule(reactContext: ReactApplicationContext) :
  NativeMathModuleSpec(reactContext) {{
  
  init {{
    // Load static library to use native methods
    System.loadLibrary("{}")
  }}

  // Declare the native method
  private external fun nativeMultiply(a: Double, b: Double): Double

  // ...

  override fun multiply(a: Double, b: Double): Double {{
    // Call the native method
    return nativeMultiply(a, b);
  }}
}}"#,
        lib_name
    );
    println!("```kt");
    highlight_code(&content, "java");
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
  s.vendored_frameworks = "ios/framework/{}.xcframework"
end"#,
        lib_name
    );
    println!("```rb");
    highlight_code(&content, "rb");
    println!();
    println!("```\n");

    let content = format!(
        r#"import "MathModule.h"
import "lib{}.h" // Add this line

@implementation MathModule
RCT_EXPORT_MODULE()

- (NSNumber *)multiply:(double *)a b:(double *)b {{
  // Call the native method
  NSNumber *result = @(multiply(a, b));
  return result;
}}

// ...

@end"#,
        lib_name
    );
    println!("```objc");
    highlight_code(&content, "mm");
    println!();
    println!("```\n");
}
