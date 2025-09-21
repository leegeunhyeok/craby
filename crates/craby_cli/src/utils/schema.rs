use craby_codegen::types::schema::Schema;
use log::error;
use owo_colors::OwoColorize;

use crate::utils::terminal::CodeHighlighter;

pub fn print_schema(schema: &Schema) -> Result<(), anyhow::Error> {
    println!("├─ Methods ({})", schema.spec.methods.len());

    let highlighter = CodeHighlighter::new();

    schema.spec.methods.iter().enumerate().try_for_each(
        |(i, method)| -> Result<(), anyhow::Error> {
            match method.as_impl_sig() {
                Ok(method_sig) => {
                    if i == schema.spec.methods.len() - 1 {
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
        },
    )?;

    // Type Aliases
    println!("├─ Alias types ({})", schema.alias_map.len());
    schema.alias_map.keys().enumerate().for_each(|(i, name)| {
        if i == schema.alias_map.len() - 1 {
            print!("│   └─ ");
        } else {
            print!("│   ├─ ");
        }
        println!("{}", name.blue());
    });
    if schema.alias_map.is_empty() {
        println!("   {}", "(None)".dimmed());
    }

    // Enums
    println!("└─ Enum types ({})", schema.enum_map.len());
    schema.enum_map.keys().enumerate().for_each(|(i, name)| {
        if i == schema.enum_map.len() - 1 {
            print!("    └─ ");
        } else {
            print!("    ├─ ");
        }
        println!("{}", name.blue());
    });
    if schema.enum_map.is_empty() {
        println!("   {}", "(None)".dimmed());
    }

    Ok(())
}
