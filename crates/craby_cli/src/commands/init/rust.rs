use std::process::Command;

use craby_build::constants::toolchain::BUILD_TARGETS;
use craby_common::env::is_rustup_installed;
use owo_colors::OwoColorize;

use crate::utils::{
    log::{success, warn},
    terminal::with_spinner,
};

pub fn setup_rust_toolchain() -> anyhow::Result<()> {
    if is_rustup_installed() {
        with_spinner("Setting up the Rust project, please wait...", |_| {
            if let Err(e) = setup_rust_targets() {
                anyhow::bail!("Failed to setup Rust project: {}", e);
            }
            Ok(())
        })?;
        success("Rust toolchain setup completed");
    } else {
        warn(&format!("Please install `rustup` to setup the Rust project for Craby\n\nVisit the Rust website: {}", "https://www.rust-lang.org/tools/install".underline()));
    }

    Ok(())
}

fn setup_rust_targets() -> anyhow::Result<()> {
    for target in BUILD_TARGETS {
        let target = target.to_str();
        let res = Command::new("rustup")
            .args(["target", "add", target])
            .output()?;

        if !res.status.success() {
            anyhow::bail!(
                "Failed to add target: {}\n{}",
                target,
                String::from_utf8_lossy(&res.stderr)
            );
        }
    }

    Ok(())
}
