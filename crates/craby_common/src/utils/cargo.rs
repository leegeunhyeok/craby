use std::process::{Command, Stdio};

pub fn cargo_version() -> Result<String, anyhow::Error> {
    let output = Command::new("cargo")
        .args(["--version"])
        .stdout(Stdio::piped())
        .output()?;

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}
