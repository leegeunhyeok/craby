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
                    method.to_rs_fn_sig().dimmed(),
                    "(excluded)".yellow()
                );
            } else if config.is_included_method(&method.name) {
                highlighter.highlight_code(&method.to_rs_fn_sig(), "rs");
            } else {
                println!("{} {}", method.name, "(not included)".dimmed());
            }
        });
    // TODO: Impl
    println!("├─ Event Emitters (0)");
    println!("│  {}", "(None)".dimmed());
    println!("├─ Type Aliases (0)");
    println!("│  {}", "(None)".dimmed());
    println!("└─ Enums (0)");
    println!("   {}", "(None)".dimmed());
}
