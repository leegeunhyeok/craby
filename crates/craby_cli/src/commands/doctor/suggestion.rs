use std::{
    cmp::Ordering,
    fmt::{self, Display},
};

use owo_colors::OwoColorize;

pub enum SuggestionType {
    Command(String),
    PlainText(Option<String>),
}

pub struct Suggestion {
    pub message: String,
    pub suggestion_type: SuggestionType,
}

impl Suggestion {
    pub fn command(message: &str, command: &str) -> Self {
        Self {
            message: message.to_string(),
            suggestion_type: SuggestionType::Command(command.to_string()),
        }
    }

    pub fn plain_text(message: &str, text: Option<&str>) -> Self {
        Self {
            message: message.to_string(),
            suggestion_type: SuggestionType::PlainText(text.map(String::from)),
        }
    }
}

impl Display for Suggestion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.suggestion_type {
            SuggestionType::Command(command) => {
                writeln!(f, "{}", format!("# {}", self.message).green())?;
                writeln!(f, "{} {}", "$".dimmed(), command)?;
            }
            SuggestionType::PlainText(text) => {
                writeln!(f, "╭─ {}", self.message)?;

                if let Some(text) = text {
                    for line in text.lines() {
                        writeln!(f, "│ {}", line)?;
                    }
                    writeln!(f, "╰─●")?;
                }
            }
        }

        Ok(())
    }
}

pub fn print_suggestions(suggestions: &mut [Suggestion]) {
    if suggestions.is_empty() {
        return;
    }

    suggestions.sort_by(|a, _| {
        if matches!(a.suggestion_type, SuggestionType::Command(..)) {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });

    println!("{}", "Suggestions\n".bold().purple());
    for suggestion in suggestions {
        println!("{}", suggestion);
    }
}
