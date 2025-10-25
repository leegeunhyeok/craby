use craby_build::setup::setup_project;
use craby_common::env::is_rustup_installed;
use owo_colors::OwoColorize;

use crate::utils::{
    log::{success, warn},
    terminal::with_spinner,
};

pub fn setup_rust_toolchain() -> anyhow::Result<()> {
    if is_rustup_installed() {
        with_spinner("Setting up the Rust project, please wait...", |_| {
            if let Err(e) = setup_project() {
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
