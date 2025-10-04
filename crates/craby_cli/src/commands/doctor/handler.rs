use std::path::PathBuf;

use craby_build::{constants::toolchain::Target, platform::android::ANDROID_TARGETS};
use craby_common::{
    constants::toolchain::TARGETS,
    env::get_installed_targets,
    utils::{android::is_gradle_configured, ios::is_podspec_configured},
};
use owo_colors::OwoColorize;

use crate::commands::doctor::assert::{assert_with_status, Status};

pub struct DoctorOptions {
    pub project_root: PathBuf,
}

pub fn perform(opts: DoctorOptions) -> anyhow::Result<()> {
    println!("\n{}", "Platform".bold().dimmed());
    assert_with_status("macOS", || {
        if std::env::consts::OS == "macos" {
            Ok(Status::Ok)
        } else {
            anyhow::bail!("Unsupported platform: {}", std::env::consts::OS);
        }
    });

    println!("\n{}", "Rust".bold().dimmed());
    let installed_targets = get_installed_targets()?;
    TARGETS.iter().for_each(|target| {
        let target_label = format!("({})", target);
        assert_with_status(
            format!("Toolchain Target {}", target_label.dimmed()).as_str(),
            || {
                if installed_targets.contains(&target.to_string()) {
                    Ok(Status::Ok)
                } else {
                    anyhow::bail!("Not installed");
                }
            },
        );
    });

    println!("\n{}", "Android".bold().dimmed());
    assert_with_status(
        format!("Environment variable: {}", "ANDROID_NDK_HOME".dimmed()).as_str(),
        || match std::env::var("ANDROID_NDK_HOME") {
            Ok(_) => Ok(Status::Ok),
            Err(e) => anyhow::bail!("Environment variable is not set: {}", e),
        },
    );

    for target in ANDROID_TARGETS {
        match target {
            Target::Android(abi) => {
                assert_with_status(
                    format!("Clang toolchain {}", format!("({})", abi.to_str()).dimmed()).as_str(),
                    || {
                        for (_, value) in abi.to_env()? {
                            if !value.try_exists()? {
                                anyhow::bail!("Clang toolchain not found: {}", abi.to_str());
                            }
                        }
                        Ok(Status::Ok)
                    },
                );
            }
            _ => unreachable!(),
        }
    }

    assert_with_status(
        format!("Build configuration {}", "(build.gradle)".dimmed()).as_str(),
        || {
            if is_gradle_configured(&opts.project_root)? {
                Ok(Status::Ok)
            } else {
                anyhow::bail!("`android/build.gradle` is not configured correctly");
            }
        },
    );

    println!("\n{}", "iOS".bold().dimmed());
    assert_with_status(
        format!("Build configuration {}", "(.podspec)".dimmed()).as_str(),
        || {
            if is_podspec_configured(&opts.project_root)? {
                Ok(Status::Ok)
            } else {
                anyhow::bail!("`android/build.gradle` is not configured correctly");
            }
        },
    );

    Ok(())
}
