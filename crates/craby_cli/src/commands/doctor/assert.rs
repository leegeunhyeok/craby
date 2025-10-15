use log::debug;
use owo_colors::OwoColorize;

const STATUS_OK: &str = "✓";
const STATUS_ERR: &str = "✗";

pub enum Status {
    Ok,
}

pub fn assert_with_status(label: &str, f: impl FnOnce() -> Result<Status, anyhow::Error>) {
    match f() {
        Ok(Status::Ok) => {
            println!("{} {}", STATUS_OK.bold().green(), label);
        }
        Err(e) => {
            println!(
                "{} {} - {}",
                STATUS_ERR.bold().red(),
                label,
                e.to_string().red()
            );
            debug!("Assertion failed: {}", e);
        }
    }
}
