use craby_codegen::types::schema::Schema;
use owo_colors::OwoColorize;

use crate::utils::terminal::highlight_code;

pub fn print_schema(schema: &Schema) {
    println!("├─ Methods ({})", schema.spec.methods.len());
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
            highlight_code(&method.to_rs_fn_sig(), "rs");
        });
    // TODO: Impl
    println!("├─ Event Emitters (0)");
    println!("│  {}", "(None)".bright_black());
    println!("├─ Type Aliases (0)");
    println!("│  {}", "(None)".bright_black());
    println!("└─ Enums (0)");
    println!("   {}", "(None)".bright_black());
}
