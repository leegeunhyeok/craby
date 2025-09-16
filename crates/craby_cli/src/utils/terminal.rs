use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};
use syntect::{
    easy::HighlightLines, highlighting::Theme, parsing::SyntaxSet, util::as_24_bit_terminal_escaped,
};
use syntect_assets::assets::HighlightingAssets;

pub fn with_spinner(
    msg: &str,
    f: impl FnOnce(&ProgressBar) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    let pb = ProgressBar::new_spinner();

    pb.set_message(msg.to_string());
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.enable_steady_tick(Duration::from_millis(120));
    f(&pb)?;
    pb.finish_and_clear();

    Ok(())
}

pub struct CodeHighlighter {
    ss: SyntaxSet,
    t: Theme,
}

impl CodeHighlighter {
    pub fn new() -> Self {
        let ast = HighlightingAssets::from_binary();
        let ss = ast.get_syntax_set().unwrap().clone();
        let t = ast.get_theme("Visual Studio Dark+").clone();

        Self { ss, t }
    }

    pub fn highlight_line(&self, line: &str, ext: &str) {
        let syntax = self.ss.find_syntax_by_extension(ext).unwrap();
        let mut h = HighlightLines::new(syntax, &self.t);
        let ranges: Vec<_> = h.highlight_line(line, &self.ss).unwrap();
        print!("{}", as_24_bit_terminal_escaped(&ranges[..], false));
        self.reset_color();
    }

    pub fn highlight_code(&self, code: &str, ext: &str) {
        for line in code.split("\n") {
            self.highlight_line(line, ext);
            println!();
        }
    }

    pub fn highlight_code_with_box(&self, code: &str, ext: &str) {
        let lines = code.split("\n").collect::<Vec<&str>>();
        let mut max_len = 0;

        lines.iter().for_each(|line| {
            if line.len() > max_len {
                max_len = line.len();
            }
        });

        max_len += 2; // For the extra padding (left, right)

        println!("╭{}╮", "─".repeat(max_len));
        for line in lines {
            // Add padding in `print!` macro, so we need to subtract 2
            let pad = max_len - line.len() - 2;
            print!("│ ");
            self.highlight_line(line, ext);
            print!("{} │", " ".repeat(pad));
            println!();
        }
        println!("╰{}╯", "─".repeat(max_len));
    }

    fn reset_color(&self) {
        print!("\x1b[0m");
    }
}
