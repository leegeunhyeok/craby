use anyhow::Result;

use crate::utils::run_command;

pub fn run() -> Result<()> {
    println!("Building...");

    run_command("cargo", &["--version"], None)?;
    run_command("yarn", &["workspace", "craby-test", "build"], None)?;
    println!("Build completed");

    Ok(())
}
