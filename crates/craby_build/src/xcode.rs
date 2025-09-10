use std::{fs, path::PathBuf};

use indoc::formatdoc;
use log::debug;

use crate::constants::{self, ios::HEADERS_PATH};
use craby_common::utils::{
    path::{binding_header_dir, ios_framework_path},
    to_header_name, to_lib_name, SanitizedString,
};

pub struct CreateXcframeworkOptions {
    pub project_root: PathBuf,
    pub header_path: PathBuf,
    pub lib_name: SanitizedString,
}

pub fn create_xcframework(opts: CreateXcframeworkOptions) -> Result<(), anyhow::Error> {
    let targets = get_ios_targets();
    let target_dir = opts.project_root.join("target");
    let lib_name = to_lib_name(&opts.lib_name);
    let xcframework = ios_framework_path(&opts.project_root, &opts.lib_name);

    if xcframework.exists() {
        fs::remove_dir_all(&xcframework)?;
        debug!("Cleaned up existing xcframework");
    }

    fs::create_dir_all(&xcframework)?;
    fs::create_dir_all(xcframework.join("ios-arm64").join(HEADERS_PATH))?;
    fs::create_dir_all(xcframework.join("ios-arm64-simulator").join(HEADERS_PATH))?;
    debug!("Created xcframework directories");

    fs::write(
        xcframework.join("Info.plist"),
        info_plist_content(&lib_name),
    )?;
    debug!("Wrote Info.plist");

    for target in targets {
        let lib_header = to_header_name(&opts.lib_name);
        let from = target_dir.join(&target).join("release").join(&lib_name);
        let from_header = binding_header_dir(&opts.project_root).join(&lib_header);
        let lib_target = if target.contains("sim") {
            "ios-arm64-simulator"
        } else {
            "ios-arm64"
        };
        let dest = xcframework.join(lib_target).join(&lib_name);

        debug!("Copying {} to {}", &lib_name, lib_target);
        debug!("Copying {} to {}", from.display(), dest.display());
        fs::copy(from, dest)?;
        fs::copy(
            from_header,
            xcframework
                .join(lib_target)
                .join(HEADERS_PATH)
                .join(lib_header),
        )?;
    }

    Ok(())
}

fn get_ios_targets() -> impl Iterator<Item = String> {
    constants::toolchain::TARGETS.iter().filter_map(|target| {
        if target.contains("ios") {
            Some(target.to_string())
        } else {
            None
        }
    })
}

fn info_plist_content(lib_name: &str) -> String {
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
                <key>HeadersPath</key>
                <string>{headers_path}</string>
                <key>LibraryIdentifier</key>
                <string>ios-arm64</string>
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
                <key>HeadersPath</key>
                <string>{headers_path}</string>
                <key>LibraryIdentifier</key>
                <string>ios-arm64-simulator</string>
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
        headers_path = HEADERS_PATH
    }
}
