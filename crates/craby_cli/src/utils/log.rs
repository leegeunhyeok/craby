use owo_colors::OwoColorize;

pub const STATUS_OK: &str = "✓";
pub const STATUS_WARN: &str = "!";

pub fn success(message: &str) {
    println!("{} {}", sym(Status::Ok), message);
}

pub fn warn(message: &str) {
    println!("{} {}", sym(Status::Warn), message);
}

pub enum Status {
    Ok,
    Warn,
}

pub fn sym(status: Status) -> String {
    match status {
        Status::Ok => STATUS_OK.bold().green().to_string(),
        Status::Warn => STATUS_WARN.bold().yellow().to_string(),
    }
}
