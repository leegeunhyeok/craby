use std::process::Command;

use super::constants;
use anyhow::Error;

pub fn setup_project() -> anyhow::Result<()> {
    setup_rust()?;

    Ok(())
}

fn setup_rust() -> anyhow::Result<()> {
    constants::toolchain::TARGETS
        .iter()
        .try_for_each(|target| {
            let res = Command::new("rustup")
                .arg("target")
                .arg("add")
                .arg(target)
                .output()?;

            if !res.status.success() {
                return Err(anyhow::anyhow!(
                    "Failed to add target: {}\n{}",
                    target,
                    String::from_utf8_lossy(&res.stderr)
                ));
            }

            Ok::<(), Error>(())
        })?;

    Ok(())
}
