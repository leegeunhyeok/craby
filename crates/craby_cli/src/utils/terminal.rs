use std::{thread::sleep, time::Duration};

use indicatif::{ProgressBar, ProgressStyle};
use syntect::{easy::HighlightLines, util::as_24_bit_terminal_escaped};
use syntect_assets::assets::HighlightingAssets;

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

pub fn highlight_code(code: &String, ext: &str) {
    let assets = HighlightingAssets::from_binary();
    let ss = assets.get_syntax_set().unwrap();
    let t = assets.get_theme("Visual Studio Dark+");
    let syntax = ss.find_syntax_by_extension(ext).unwrap();

    for line in code.split("\n") {
        let mut h = HighlightLines::new(syntax, t);
        let ranges: Vec<_> = h.highlight_line(line, &ss).unwrap();
        print!("{}", as_24_bit_terminal_escaped(&ranges[..], false));
        println!();
    }

    // Reset color
    print!("\x1b[0m");
}
