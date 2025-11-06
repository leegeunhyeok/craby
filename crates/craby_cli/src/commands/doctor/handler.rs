use std::path::PathBuf;

use craby_build::{constants::toolchain::Target, platform::android::ANDROID_TARGETS};
use craby_common::{
    constants::toolchain::TARGETS,
    env::get_installed_targets,
    utils::{
        android::is_gradle_configured,
        ios::{is_podspec_configured, is_xcode_cli_tools_installed},
    },
};
use indoc::formatdoc;
use owo_colors::OwoColorize;

use crate::commands::doctor::{
    assert::{assert_with_status, Status},
    suggestion::{print_suggestions, Suggestion},
};

pub struct DoctorOptions {
    pub project_root: PathBuf,
}

pub fn perform(opts: DoctorOptions) -> anyhow::Result<()> {
    println!("\n{}", "Platform".bold().dimmed());
    let mut passed = true;
    let mut suggestions = Vec::new();

    assert_with_status("macOS", || {
        if std::env::consts::OS == "macos" {
            Ok(Status::Ok)
        } else {
            passed &= false;
            anyhow::bail!("Unsupported platform: {}", std::env::consts::OS);
        }
    });

    println!("\n{}", "Rust".bold().dimmed());
    let installed_targets = get_installed_targets()?;
    TARGETS.iter().for_each(|target| {
        let target_label = format!("({target})");
        assert_with_status(
            &format!("Toolchain Target {}", target_label.dimmed()),
            || {
                if installed_targets.contains(&target.to_string()) {
                    Ok(Status::Ok)
                } else {
                    passed &= false;
                    suggestions.push(Suggestion::command(
                        &format!("Install '{}' target with rustup", target),
                        &format!("rustup target install {target}"),
                    ));
                    anyhow::bail!("Not installed");
                }
            },
        );
    });

    println!("\n{}", "Android".bold().dimmed());
    assert_with_status(
        &format!("Environment variable: {}", "ANDROID_NDK_HOME".dimmed()),
        || match std::env::var("ANDROID_NDK_HOME") {
            Ok(_) => Ok(Status::Ok),
            Err(e) => {
                passed &= false;
                suggestions.push(Suggestion::plain_text(
                    &format!("Check {} path is set correctly", "$ANDROID_NDK_HOME".yellow()),
                    Some(&formatdoc! {
                        r#"
                        If Android NDK is not installed, please install it from the following link:
                        {link}"#,
                        link = "https://developer.android.com/ndk/downloads".dimmed().underline()
                    }),
                ));
                anyhow::bail!("Environment variable is not set: {}", e);
            }
        },
    );

    for target in ANDROID_TARGETS {
        match target {
            Target::Android(abi) => {
                assert_with_status(
                    &format!("Clang toolchain {}", format!("({abi})").dimmed()),
                    || {
                        for (_, value) in abi.to_env()? {
                            if !value.try_exists()? {
                                passed &= false;
                                anyhow::bail!("Clang toolchain not found: {abi}");
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
        &format!("Build configuration {}", "(build.gradle)".dimmed()),
        || {
            if is_gradle_configured(&opts.project_root)? {
                Ok(Status::Ok)
            } else {
                passed &= false;
                suggestions.push(Suggestion::plain_text(
                    "Run `crabygen codegen` to fix this issue",
                    None,
                ));
                anyhow::bail!("`android/build.gradle` is not configured correctly");
            }
        },
    );

    println!("\n{}", "iOS".bold().dimmed());
    assert_with_status("XCode Command Line Tools", || {
        if is_xcode_cli_tools_installed()? {
            Ok(Status::Ok)
        } else {
            passed &= false;
            suggestions.push(Suggestion::command(
                "Install XCode Command Line Tools",
                "xcode-select --install",
            ));
            anyhow::bail!("XCode Command Line Tools is not installed");
        }
    });
    assert_with_status(
        &format!("Build configuration {}", "(.podspec)".dimmed()),
        || {
            if is_podspec_configured(&opts.project_root)? {
                Ok(Status::Ok)
            } else {
                passed &= false;
                anyhow::bail!("`.podspec` is not configured correctly");
            }
        },
    );

    if !passed {
        println!();
        print_suggestions(&mut suggestions);
        anyhow::bail!("Some required configurations are not configured correctly");
    }

    Ok(())
}
