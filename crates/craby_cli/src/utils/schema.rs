use craby_codegen::types::schema::Schema;
use craby_common::config::CompleteCrabyConfig;
use owo_colors::OwoColorize;

use crate::utils::terminal::CodeHighlighter;

pub fn print_schema(schema: &Schema, config: &CompleteCrabyConfig) {
    println!("├─ Methods ({})", schema.spec.methods.len());

    let highlighter = CodeHighlighter::new();

    schema
        .spec
        .methods
        .iter()
        .enumerate()
        .for_each(|(i, method)| {
            if i == schema.spec.methods.len() - 1 {
                print!("│   └─ ");
            } else {
                print!("│   ├─ ");
            }

            if config.is_excluded_method(&method.name) {
                println!(
                    "{} {}",
                    method.to_sig().dimmed(),
                    "(excluded)".yellow()
                );
            } else if config.is_included_method(&method.name) {
                highlighter.highlight_code(&method.to_sig(), "rs");
            } else {
                println!("{} {}", method.name, "(not included)".dimmed());
            }
        });

    // Type Aliases
    println!("├─ Type Aliases ({})", schema.alias_map.len());
    schema.alias_map.keys().enumerate().for_each(|(i, name)| {
        if i == schema.alias_map.len() - 1 {
            print!("│   └─ ");
        } else {
            print!("│   ├─ ");
        }
        println!("{}", name.blue());
    });

    // TODO: Impl
    // Event Emitters
    println!("├─ Event Emitters (0)");
    println!("│  {}", "(None)".dimmed());

    // Enums
    println!("└─ Enums (0)");
    println!("   {}", "(None)".dimmed());
}
