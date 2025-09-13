use std::{fs, path::PathBuf};

use crate::{
    cargo::artifact::{ArtifactType, Artifacts},
    constants::{ios::Identifier, toolchain::Target},
};
use craby_common::{
    config::CompleteCrabyConfig, constants::{dest_lib_name, lib_base_name}, utils::string::SanitizedString,
};
use indoc::formatdoc;
use log::debug;

pub fn crate_libs<'a>(config: &'a CompleteCrabyConfig) -> Result<(), anyhow::Error> {
    let ios_base_path = ios_base_path(&config.project_root);

    if ios_base_path.exists() {
        fs::remove_dir_all(&ios_base_path)?;
        debug!("Cleaned up existing iOS base directory");
    }

    let xcframework_path = create_xcframework(&config)?;

    for target in [
        Target::Ios(Identifier::Arm64),
        Target::Ios(Identifier::Arm64Simulator),
    ] {
        if let Target::Ios(identifier) = &target {
            let artifacts = Artifacts::get_artifacts(config, &target)?;
            let identifier = identifier.to_str();

            // {ios_base_path}/src
            artifacts.copy_to(ArtifactType::Src, &ios_base_path.join("src"))?;

            // {ios_base_path}/include
            artifacts.copy_to(ArtifactType::Header, &ios_base_path.join("include"))?;

            // {ios_base_path}/framework/lib{lib_name}.xcframework/{identifier}
            artifacts.copy_to(ArtifactType::Lib, &xcframework_path.join(identifier))?;
        } else {
            unreachable!();
        }
    }

    Ok(())
}

fn create_xcframework(config: &CompleteCrabyConfig) -> Result<PathBuf, anyhow::Error> {
    let name = SanitizedString::from(&config.project.name);
    let dest_lib_name = dest_lib_name(&name);
    let lib_base_name = lib_base_name(&name);
    let info_plist_content = info_plist_content(&dest_lib_name);
    let framework_path = ios_base_path(&config.project_root).join("framework");
    let xcframework_path =
        framework_path.join(format!("lib{}.xcframework", lib_base_name.to_string()));

    if xcframework_path.exists() {
        fs::remove_dir_all(&xcframework_path)?;
        debug!("Cleaned up existing iOS xcframework");
    }

    fs::create_dir_all(&xcframework_path)?;

    let info_plist_path = xcframework_path.join("Info.plist");
    fs::write(info_plist_path, info_plist_content)?;

    Ok(xcframework_path)
}

fn info_plist_content(lib_name: &String) -> String {
    formatdoc! {
        r#"<?xml version="1.0" encoding="UTF-8"?>
        <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
        <plist version="1.0">
        <dict>
            <key>AvailableLibraries</key>
            <array>
                <dict>
                    <key>BinaryPath</key>
                    <string>{lib_name}</string>
                    <key>LibraryIdentifier</key>
                    <string>{lib_identifier}</string>
                    <key>LibraryPath</key>
                    <string>{lib_name}</string>
                    <key>SupportedArchitectures</key>
                    <array>
                        <string>arm64</string>
                    </array>
                    <key>SupportedPlatform</key>
                    <string>ios</string>
                </dict>
                <dict>
                    <key>BinaryPath</key>
                    <string>{lib_name}</string>
                    <key>LibraryIdentifier</key>
                    <string>{lib_sim_identifier}</string>
                    <key>LibraryPath</key>
                    <string>{lib_name}</string>
                    <key>SupportedArchitectures</key>
                    <array>
                        <string>arm64</string>
                    </array>
                    <key>SupportedPlatform</key>
                    <string>ios</string>
                    <key>SupportedPlatformVariant</key>
                    <string>simulator</string>
                </dict>
            </array>
            <key>CFBundlePackageType</key>
            <string>XFWK</string>
            <key>XCFrameworkFormatVersion</key>
            <string>1.0</string>
        </dict>
        </plist>"#,
        lib_name = lib_name,
        lib_identifier = Identifier::Arm64.to_str(),
        lib_sim_identifier = Identifier::Arm64Simulator.to_str(),
    }
}

fn ios_base_path(project_root: &PathBuf) -> PathBuf {
    project_root.join("ios")
}
