use std::process::Command;

use crate::constants::{android::Abi, ios::Identifier, toolchain::Target};

pub fn setup_project() -> anyhow::Result<()> {
    setup_rust()?;

    Ok(())
}

fn setup_rust() -> anyhow::Result<()> {
    for target in [
        Target::Android(Abi::Arm64V8a),
        Target::Android(Abi::ArmeAbiV7a),
        Target::Android(Abi::X86_64),
        Target::Android(Abi::X86),
        Target::Ios(Identifier::Arm64),
        Target::Ios(Identifier::Arm64Simulator),
    ] {
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
