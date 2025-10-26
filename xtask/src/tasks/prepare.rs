use anyhow::Result;
use craby_build::constants::toolchain::BUILD_TARGETS;

use crate::utils::run_command;

pub const EXCLUDE_PACKAGE_NAMES: [&str; 3] = ["craby-test", "craby-0.76", "craby-0.80"];

pub fn run(opt: Option<&str>) -> Result<()> {
    let is_ts = opt.is_some_and(|o| o == "--ts");

    println!(
        "Preparing for {} integrations...",
        if is_ts { "TypeScript" } else { "Rust" }
    );

    // Setup toolchain targets for Rust
    if !is_ts {
        println!("Setting up toolchain targets...");
        run_command("cargo", &["--version"], None)?;

        for target in BUILD_TARGETS {
            println!("Installing target: {}", target.to_str());
            run_command("rustup", &["target", "install", target.to_str()], None)?;
        }
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
            format!("{{{}}}", EXCLUDE_PACKAGE_NAMES.join(",")).as_str(),
            "run",
            "build",
        ],
        None,
    )?;

    // Build JS bundle only for `craby-test`
    // because `build (craby build)` command requires macOS and Xcode
    run_command("yarn", &["workspace", "craby-test", "build:js"], None)?;

    println!("Prepare completed");

    Ok(())
}
