use std::path::PathBuf;

use anyhow::Result;
use craby_build::constants::toolchain::BUILD_TARGETS;

use crate::utils::run_command;

const EXCLUDE_PACKAGE_NAMES: [&str; 3] = ["craby-test", "craby-0.76", "craby-0.80"];

pub fn run() -> Result<()> {
    println!("Preparing...");

    let craby_test_path = PathBuf::from(".").join("examples").join("craby-test");
    let craby_test_path = craby_test_path.to_str().unwrap();

    run_command("cargo", &["--version"], Some(craby_test_path))?;

    for target in BUILD_TARGETS {
        println!("Installing target: {}", target.to_str());
        run_command(
            "rustup",
            &["target", "install", target.to_str()],
            Some(craby_test_path),
        )?;
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
