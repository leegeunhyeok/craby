use log::debug;
use owo_colors::OwoColorize;

const STATUS_OK: &str = "✓";
const STATUS_ERR: &str = "✗";

pub fn assert_with_status(label: &str, f: impl FnOnce() -> Result<(), anyhow::Error>) {
    match f() {
        Ok(_) => {
            println!("{} {}", STATUS_OK.bold().green(), label);
        }
        Err(e) => {
            println!("{} {} - {}", STATUS_ERR.bold().red(), label, e.to_string());
            debug!("Assertion failed: {}", e);
        }
    }
}
