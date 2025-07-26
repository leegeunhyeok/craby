use std::{fs, path::PathBuf};

use craby_codegen::types::schema::{AndroidConfig, LibraryConfig};
use craby_common::{
    constants::toolchain::TARGETS,
    env::{get_installed_targets, is_cargo_ndk_installed, is_xcode_installed},
    utils::{android::is_gradle_configured, ios::is_podspec_configured},
};
use log::debug;
use owo_colors::OwoColorize;

use crate::commands::doctor::assert::{assert_with_status, Status};

pub struct DoctorOptions {
    pub project_root: PathBuf,
}

pub fn r#impl(opts: DoctorOptions) -> anyhow::Result<()> {
    let package_json = fs::read_to_string(opts.project_root.join("package.json"))?;
    let package_json = serde_json::from_str::<serde_json::Value>(&package_json)?;

    println!("\n{}", "Common".bold().dimmed());
    assert_with_status("TurboModule Configuration", || {
        match package_json.get("codegenConfig") {
            Some(cfg) => match serde_json::from_str::<LibraryConfig>(&cfg.to_string()) {
                Ok(lib_cfg) => match lib_cfg {
                    LibraryConfig {
                        js_srcs_dir: Some(_),
                        android:
                            Some(AndroidConfig {
                                java_package_name: Some(_),
                            }),
                        ..
                    } => Ok(Status::Ok),
                    _ => Err(anyhow::anyhow!(
                        "`codegenConfig.jsSrcsDir` and `codegenConfig.android.javaPackageName` are required"
                    )),
                },
                Err(e) => {
                    debug!("Parse error: {}", e);
                    return Err(anyhow::anyhow!("Invalid `codegenConfig` value"));
                }
            },
            None => Err(anyhow::anyhow!(
                "`codegenConfig` field not found in the `package.json`"
            )),
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
                    Err(anyhow::anyhow!("Not installed"))
                }
            },
        );
    });

    println!("\n{}", "Android".bold().dimmed());
    assert_with_status("ANDROID_HOME", || match std::env::var("ANDROID_HOME") {
        Ok(_) => Ok(Status::Ok),
        Err(e) => Err(anyhow::anyhow!(
            "`ANDROID_HOME` environment variable is not set: {}",
            e
        )),
    });
    assert_with_status("cargo-ndk", || {
        if is_cargo_ndk_installed() {
            Ok(Status::Ok)
        } else {
            Err(anyhow::anyhow!("`cargo-ndk` is not installed"))
        }
    });
    assert_with_status("Build configuration", || {
        if is_gradle_configured(&opts.project_root)? {
            Ok(Status::Ok)
        } else {
            Err(anyhow::anyhow!(
                "`android/build.gradle` is not configured correctly"
            ))
        }
    });

    println!("\n{}", "iOS".bold().dimmed());
    assert_with_status("XCode", || {
        if is_xcode_installed() {
            Ok(Status::Ok)
        } else {
            Ok(Status::Warn(format!(
                "`xcodebuild` command not found. {}",
                "(The xcframework will be generated manually instead)".dimmed()
            )))
        }
    });
    assert_with_status("Build configuration", || {
        if is_podspec_configured(&opts.project_root)? {
            Ok(Status::Ok)
        } else {
            Err(anyhow::anyhow!(
                "`<LibraryName>.podspec` is not configured correctly"
            ))
        }
    });

    Ok(())
}
