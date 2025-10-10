use anyhow::Result;
use craby_build::constants::toolchain::BUILD_TARGETS;

use crate::utils::run_command;

const EXAMPLE_APP_NAME: [&str; 1] = ["craby-0.80"];

pub fn run() -> Result<()> {
    println!("Preparing...");

    for target in BUILD_TARGETS {
        println!("Installing target: {}", target.to_str());
        run_command("rustup", &["target", "install", target.to_str()], None)?;
    }

    println!("Building packages...");
    run_command(
        "yarn",
        &[
            "workspaces",
            "foreach",
            "--all",
            "--topological-dev",
            "--exclude",
            EXAMPLE_APP_NAME.join(",").as_str(),
            "run",
            "build",
        ],
        None,
    )?;

    println!("Prepare completed");

    Ok(())
}
