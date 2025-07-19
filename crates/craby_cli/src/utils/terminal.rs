use std::{thread::sleep, time::Duration};

use indicatif::{ProgressBar, ProgressStyle};

pub fn with_spinner(msg: &str, f: impl FnOnce() -> anyhow::Result<()>) -> anyhow::Result<()> {
    let pb = ProgressBar::new_spinner();

    pb.set_message(msg.to_string());
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .expect("Invalid template"),
    );
    pb.enable_steady_tick(Duration::from_millis(120));
    f()?;
    sleep(Duration::from_secs(3));
    pb.finish_and_clear();

    Ok(())
}
