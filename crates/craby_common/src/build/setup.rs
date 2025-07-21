use std::process::Command;

use anyhow::Error;

use crate::constants;

pub fn setup_project() -> anyhow::Result<()> {
    setup_rust()?;
    setup_ndk()?;

    Ok(())
}

fn setup_rust() -> anyhow::Result<()> {
    constants::toolchain::TARGETS
        .iter()
        .try_for_each(|target| {
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

            Ok::<(), Error>(())
        })?;

    let res = Command::new("cargo")
        .args(["install", "cargo-ndk"])
        .output()?;

    if !res.status.success() {
        anyhow::bail!(
            "Failed to install cargo-ndk\n{}",
            String::from_utf8_lossy(&res.stderr)
        );
    }

    Ok(())
}

fn setup_ndk() -> anyhow::Result<()> {
    let res = Command::new("cargo")
        .args(["install", "cargo-ndk"])
        .output()?;

    if !res.status.success() {
        anyhow::bail!(
            "Failed to install cargo-ndk\n{}",
            String::from_utf8_lossy(&res.stderr)
        );
    }

    Ok(())
}
