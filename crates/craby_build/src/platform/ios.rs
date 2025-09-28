use std::{fs, path::PathBuf};

use crate::{
    cargo::artifact::{ArtifactType, Artifacts},
    constants::{ios::Identifier, toolchain::Target},
    cxx::replace_cxx_header,
};

use craby_common::{
    config::CompleteCrabyConfig,
    constants::{dest_lib_name, ios_base_path, lib_base_name},
    utils::string::SanitizedString,
};
use indoc::formatdoc;

const IOS_TARGETS: [Target; 2] = [
    Target::Ios(Identifier::Arm64),
    Target::Ios(Identifier::Arm64Simulator),
];

pub fn crate_libs<'a>(config: &'a CompleteCrabyConfig) -> Result<(), anyhow::Error> {
    let ios_base_path = ios_base_path(&config.project_root);
    let xcframework_path = create_xcframework(&config)?;

    for target in IOS_TARGETS {
        if let Target::Ios(identifier) = &target {
            let artifacts = Artifacts::get_artifacts(config, &target)?;
            let identifier = identifier.to_str();

            // ios/src
            artifacts.copy_to(ArtifactType::Src, &ios_base_path.join("src"))?;

            // ios/include
            artifacts.copy_to(ArtifactType::Header, &ios_base_path.join("include"))?;

            // ios/framework/lib{lib_name}.xcframework/{identifier}
            artifacts.copy_to(ArtifactType::Lib, &xcframework_path.join(identifier))?;
        } else {
            unreachable!();
        }
    }

    let signal_path = ios_base_path.join("include").join("signals.h");
    if signal_path.exists() {
        replace_cxx_header(&signal_path)?;
    }

    Ok(())
}

fn create_xcframework(config: &CompleteCrabyConfig) -> Result<PathBuf, anyhow::Error> {
    let name = SanitizedString::from(&config.project.name);
    let lib_base_name = lib_base_name(&name);
    let info_plist_content = info_plist(&config.project.name);
    let framework_path = ios_base_path(&config.project_root).join("framework");
    let xcframework_path =
        framework_path.join(format!("lib{}.xcframework", lib_base_name.to_string()));

    fs::create_dir_all(&xcframework_path)?;

    let info_plist_path = xcframework_path.join("Info.plist");
    fs::write(info_plist_path, info_plist_content)?;

    Ok(xcframework_path)
}

pub fn info_plist(name: &String) -> String {
    let lib_name = dest_lib_name(&SanitizedString::from(name));

    formatdoc! {
        r#"
        <?xml version="1.0" encoding="UTF-8"?>
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
