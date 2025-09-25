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
    println!("├─ Alias types ({})", schema.alias_map.len());
    schema.alias_map.iter().enumerate().for_each(|(i, obj)| {
        if i == schema.alias_map.len() - 1 {
            print!("│   └─ ");
        } else {
            print!("│   ├─ ");
        }
        println!("{}", obj.as_object().unwrap().name.blue());
    });
    if schema.alias_map.is_empty() {
        println!("│  {}", "(None)".dimmed());
    }

    // Enums
    println!("└─ Enum types ({})", schema.enum_map.len());
    schema
        .enum_map
        .iter()
        .enumerate()
        .for_each(|(i, enum_spec)| {
            if i == schema.enum_map.len() - 1 {
                print!("    └─ ");
            } else {
                print!("    ├─ ");
            }
            println!("{}", enum_spec.as_enum().unwrap().name.blue());
        });
    if schema.enum_map.is_empty() {
        println!("   {}", "(None)".dimmed());
    }

    Ok(())
}
