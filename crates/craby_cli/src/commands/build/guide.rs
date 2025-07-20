use owo_colors::OwoColorize;
use syntect::{
    easy::HighlightLines, highlighting::ThemeSet, parsing::SyntaxSet,
    util::as_24_bit_terminal_escaped,
};

pub fn print_guide(lib_name: &String) {
    print_usage(lib_name);
}

fn print_usage(lib_name: &String) {
    println!("\nAndroid usage:\n");
    println!(
        "Open `{}` file and add the following line:\n",
        "android/build.gradle".underline()
    );
    let content = r#"android {
  // ...

  sourceSets {
    main {
      // ...
      jniLibs.srcDirs += ["src/main/jniLibs"]
    }
  }
}
"#;
    highlight_code(&content.to_string(), "gradle");

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
}}
"#,
        lib_name
    );
    highlight_code(&content, "java");

    println!("iOS usage:\n");
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
    highlight_code(&content, "rb");
    println!("\n");
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

@end
"#,
        lib_name
    );
    highlight_code(&content, "mm");
}

fn highlight_code(code: &String, ext: &str) {
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let t = &ts.themes["base16-ocean.dark"];
    let syntax = ss.find_syntax_by_extension(ext).unwrap();
    let mut h = HighlightLines::new(syntax, t);

    for line in code.split("\n") {
        let ranges: Vec<_> = h.highlight_line(line, &ss).unwrap();
        print!("{}", as_24_bit_terminal_escaped(&ranges[..], false));
        println!();
    }
}
