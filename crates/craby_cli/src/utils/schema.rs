use craby_codegen::types::Schema;
use log::error;
use owo_colors::OwoColorize;

use crate::utils::terminal::CodeHighlighter;

pub fn print_schema(schema: &Schema) -> Result<(), anyhow::Error> {
    println!("├─ Methods ({})", schema.methods.len());

    let highlighter = CodeHighlighter::new();

    schema
        .methods
        .iter()
        .enumerate()
        .try_for_each(|(i, method)| -> Result<(), anyhow::Error> {
            match method.as_impl_sig() {
                Ok(method_sig) => {
                    if i == schema.methods.len() - 1 {
                        print!("│   └─ ");
                    } else {
                        print!("│   ├─ ");
                    }
                    highlighter.highlight_code(&method_sig, "rs");
                }
                Err(e) => {
                    error!("Failed to get method signature: {}", method.name);
                    return Err(e);
                }
            }

            Ok(())
        })?;

    // Type Aliases
    let alias_count = schema.aliases.len();
    println!("├─ Alias types ({})", alias_count);
    schema.aliases.iter().enumerate().for_each(|(i, obj)| {
        if i == alias_count - 1 {
            print!("│   └─ ");
        } else {
            print!("│   ├─ ");
        }
        println!("{}", obj.as_object().unwrap().name.blue());
    });
    if schema.aliases.is_empty() {
        println!("│  {}", "(None)".dimmed());
    }

    // Enums
    let enum_count = schema.enums.len();
    println!("└─ Enum types ({})", enum_count);
    schema.enums.iter().enumerate().for_each(|(i, enum_spec)| {
        if i == enum_count - 1 {
            print!("    └─ ");
        } else {
            print!("    ├─ ");
        }
        println!("{}", enum_spec.as_enum().unwrap().name.blue());
    });
    if schema.enums.is_empty() {
        println!("   {}", "(None)".dimmed());
    }

    Ok(())
}
