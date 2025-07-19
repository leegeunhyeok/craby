use std::{io::Write, sync::Once};

use env_logger::Builder;
use log::{Level, LevelFilter};
use owo_colors::OwoColorize;

static INIT: Once = Once::new();

fn to_level_str(level: Level) -> String {
    match level {
        Level::Trace => "TRACE".bright_black().to_string(),
        Level::Debug => "DEBUG".bright_black().to_string(),
        Level::Info => "INFO".cyan().to_string(),
        Level::Warn => "WARN".yellow().to_string(),
        Level::Error => "ERROR".red().to_string(),
    }
}

pub fn init(level_filter: Option<LevelFilter>) {
    INIT.call_once(|| {
        let level_filter = level_filter.unwrap_or(LevelFilter::Info);
        let is_debug = level_filter == LevelFilter::Debug || level_filter == LevelFilter::Trace;
        let mut builder = Builder::new();
        let mut builder = builder.filter_level(level_filter);

        if !is_debug {
            builder = builder.format(|buf, record| {
                writeln!(
                    buf,
                    "{level} {message}",
                    level = to_level_str(record.level()),
                    message = record.args()
                )
            })
        }

        builder.init();
    });
}
