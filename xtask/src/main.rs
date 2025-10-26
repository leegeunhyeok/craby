use anyhow::Result;
use std::env;

mod tasks;
mod utils;

fn main() -> Result<()> {
    let task = env::args().nth(1);
    let opt = env::args().nth(2);

    match task.as_deref() {
        Some("version") => tasks::version::run(),
        Some("publish") => tasks::publish::run(),
        Some("prepare") => tasks::prepare::run(opt.as_deref()),
        Some("build") => tasks::build::run(),
        _ => {
            eprintln!("Usage: cargo xtask [version|publish]");
            std::process::exit(1);
        }
    }
}
