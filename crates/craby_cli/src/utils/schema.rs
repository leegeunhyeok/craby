use craby_codegen::types::Schema;
use owo_colors::OwoColorize;

use crate::utils::terminal::CodeHighlighter;

pub fn print_schema(schema: &Schema) -> Result<(), anyhow::Error> {
    println!("├─ Methods ({})", schema.methods.len());

    let highlighter = CodeHighlighter::new();

    for (i, method) in schema.methods.iter().enumerate() {
        match method.try_into_impl_sig() {
            Ok(method_sig) => {
                let is_last = i == schema.methods.len() - 1;
                let branch = if is_last { "└─" } else { "├─" };
                print!("│   {} ", branch);
                highlighter.highlight_code(&method_sig, "rs");
            }
            Err(_) => anyhow::bail!("Failed to get method signature: {}", method.name),
        }
    }

    // Type Aliases
    let alias_count = schema.aliases.len();
    println!("├─ Alias types ({})", alias_count);
    schema.aliases.iter().enumerate().for_each(|(i, obj_spec)| {
        let is_last = i == alias_count - 1;
        let branch = if is_last { "└─" } else { "├─" };
        println!(
            "│   {} {}",
            branch,
            obj_spec.as_object().unwrap().name.blue()
        );
    });
    if schema.aliases.is_empty() {
        println!("│  {}", "(None)".dimmed());
    }

    // Enums
    let enum_count = schema.enums.len();
    println!("├─ Enum types ({})", enum_count);
    schema.enums.iter().enumerate().for_each(|(i, enum_spec)| {
        let is_last = i == enum_count - 1;
        let branch = if is_last { "└─" } else { "├─" };
        println!(
            "│   {} {}",
            branch,
            enum_spec.as_enum().unwrap().name.blue()
        );
    });
    if schema.enums.is_empty() {
        println!("   {}", "(None)".dimmed());
    }

    // Signals
    let signal_count = schema.signals.len();
    println!("└─ Signals ({})", signal_count);
    schema
        .signals
        .iter()
        .enumerate()
        .for_each(|(i, signal_spec)| {
            let is_last = i == signal_count - 1;
            let branch = if is_last { "└─" } else { "├─" };
            println!("    {} {}", branch, signal_spec.name.blue());
        });
    if schema.signals.is_empty() {
        println!("   {}", "(None)".dimmed());
    }

    Ok(())
}
