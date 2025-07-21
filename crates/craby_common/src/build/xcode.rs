use std::{fs, path::PathBuf, process::Command};

use log::debug;

use crate::{
    constants,
    env::Platform,
    utils::{
        path::{binding_header_dir, ios_framework_path},
        to_lib_name,
    },
};

pub struct CreateXcframeworkOptions {
    pub project_root: PathBuf,
    pub header_path: PathBuf,
    pub lib_name: String,
}

#[cfg(target_os = "macos")]
pub fn create_xcframework(opts: CreateXcframeworkOptions) -> Result<(), anyhow::Error> {
    use crate::{
        env::{is_xcode_installed, Platform},
        utils::{
            path::{crate_target_dir, ios_framework_path},
            to_lib_name,
        },
    };

    if is_xcode_installed() {
        let xcframework_path = ios_framework_path(&opts.project_root, &opts.lib_name);

        if xcframework_path.exists() {
            fs::remove_dir_all(&xcframework_path)?;
            debug!("Cleaned up existing xcframework");
        }

        let mut cmd = Command::new("xcodebuild");
        let cmd = cmd.args([
            "-create-xcframework",
            "-output",
            xcframework_path.to_str().unwrap(),
        ]);

        get_ios_targets().for_each(|target| {
            let lib = crate_target_dir(&opts.project_root, &target)
                .join(to_lib_name(&opts.lib_name, Platform::iOS));

            cmd.arg("-library")
                .args(["-library", lib.to_str().unwrap()])
                .args(["-headers", opts.header_path.to_str().unwrap()]);
        });

        // xcodebuild -create-xcframework \
        //   -output <output_dir>/<lib_name>.xcframework \
        //   -library <lib_path_1> \
        //   -headers <header_path>
        //   -library <lib_path_2> \
        //   -headers <header_path>
        let res = cmd.output()?;

        if !res.status.success() {
            anyhow::bail!(
                "Failed to create Xcode framework: {}",
                String::from_utf8_lossy(&res.stderr)
            );
        }
    } else {
        debug!("xcodebuild: command not found. falling back to manual xcframework generation");
        generate_xcframework(opts)?;
    }

    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn create_xcframework(opts: CreateXcframeworkOptions) -> Result<(), anyhow::Error> {
    generate_xcframework(opts)?;
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

fn generate_xcframework(opts: CreateXcframeworkOptions) -> Result<(), anyhow::Error> {
    let targets = get_ios_targets();
    let headers_path = "Headers";
    let target_dir = opts.project_root.join("target");
    let xcframework = ios_framework_path(&opts.project_root, &opts.lib_name);

    if xcframework.exists() {
        fs::remove_dir_all(&xcframework)?;
        debug!("Cleaned up existing xcframework");
    }

    fs::create_dir_all(&xcframework)?;
    fs::create_dir_all(xcframework.join("ios-arm64").join(headers_path))?;
    fs::create_dir_all(xcframework.join("ios-arm64-simulator").join(headers_path))?;
    debug!("Created xcframework directories");

    fs::write(
        xcframework.join("Info.plist"),
        info_plist_content(&opts.lib_name, &headers_path),
    )?;
    debug!("Wrote Info.plist");

    for target in targets {
        let lib: String = format!("lib{}.a", opts.lib_name);
        let lib_header = format!("lib{}.h", opts.lib_name);
        let from = target_dir.join(&target).join("release").join(&lib);
        let from_header = binding_header_dir(&opts.project_root).join(&lib_header);

        let lib_target = if target.contains("sim") {
            "ios-arm64-simulator"
        } else {
            "ios-arm64"
        };

        debug!("Copying {} to {}", &lib, lib_target);
        fs::copy(from, xcframework.join(lib_target).join(&lib))?;
        fs::copy(
            from_header,
            xcframework
                .join(lib_target)
                .join(headers_path)
                .join(lib_header),
        )?;
    }

    Ok(())
}

fn info_plist_content(lib_name: &str, headers_path: &str) -> String {
    let lib_value = format!(
        "      <string>{}</string>",
        to_lib_name(&lib_name.to_string(), Platform::iOS)
    );
    let headers_value = format!("      <string>{}</string>", headers_path);

    [
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>",
        "<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">",
        "<plist version=\"1.0\">",
        "<dict>",
        "  <key>AvailableLibraries</key>",
        "  <array>",
        "    <dict>",
        "      <key>BinaryPath</key>",
        lib_value.as_str(),
        "      <key>HeadersPath</key>",
        headers_value.as_str(),
        "      <key>LibraryIdentifier</key>",
        "      <string>ios-arm64</string>",
        "      <key>LibraryPath</key>",
        lib_value.as_str(),
        "      <key>SupportedArchitectures</key>",
        "      <array>",
        "        <string>arm64</string>",
        "      </array>",
        "      <key>SupportedPlatform</key>",
        "      <string>ios</string>",
        "    </dict>",
        "    <dict>",
        "      <key>BinaryPath</key>",
        lib_value.as_str(),
        "      <key>HeadersPath</key>",
        headers_value.as_str(),
        "      <key>LibraryIdentifier</key>",
        "      <string>ios-arm64-simulator</string>",
        "      <key>LibraryPath</key>",
        lib_value.as_str(),
        "      <key>SupportedArchitectures</key>",
        "      <array>",
        "        <string>arm64</string>",
        "      </array>",
        "      <key>SupportedPlatform</key>",
        "      <string>ios</string>",
        "      <key>SupportedPlatformVariant</key>",
        "      <string>simulator</string>",
        "    </dict>",
        "  </array>",
        "  <key>CFBundlePackageType</key>",
        "  <string>XFWK</string>",
        "  <key>XCFrameworkFormatVersion</key>",
        "  <string>1.0</string>",
        "</dict>",
        "</plist>",
    ]
    .join("\n")
}
