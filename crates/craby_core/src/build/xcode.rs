use std::{fs, path::PathBuf, process::Command};

use craby_common::constants;
use log::{debug, warn};

pub struct CreateXcframeworkOptions {
    project_root: PathBuf,
    header_path: PathBuf,
    lib_name: String,
}

#[cfg(target_os = "macos")]
pub fn create_xcframework(opts: CreateXcframeworkOptions) -> Result<(), anyhow::Error> {
    if can_use_xcode() {
        let mut cmd = Command::new("xcodebuild");
        let cmd = cmd.arg("-create-xcframework").arg("-output").arg(
            opts.project_root
                .join("ios")
                .join("framework")
                .join(format!("lib{}.xcframework", opts.lib_name)),
        );

        get_ios_targets().for_each(|target| {
            let lib = opts
                .project_root
                .join("target")
                .join(target)
                .join("release")
                .join(format!("lib{}.a", opts.lib_name));

            cmd.arg("-library")
                .arg(lib)
                .arg("-headers")
                .arg(opts.header_path.clone());
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
        warn!("xcodebuild: command not found. falling back to manual xcframework generation");
        generate_xcframework(opts)?;
    }

    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn create_xcframework(opts: CreateXcframeworkOptions) -> Result<(), anyhow::Error> {
    generate_xcframework(opts)?;
}

fn can_use_xcode() -> bool {
    match Command::new("xcodebuild").arg("-version").status() {
        Ok(status) => status.success(),
        Err(_) => false,
    }
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
    let xcframework = opts
        .project_root
        .join("ios")
        .join("framework")
        .join(format!("lib{}.xcframework", opts.lib_name));

    if xcframework.exists() {
        fs::remove_dir_all(&xcframework)?;
        debug!("Cleaned up existing xcframework");
    }

    fs::create_dir_all(&xcframework)?;
    fs::create_dir_all(xcframework.join("ios-arm64").join(headers_path))?;
    fs::create_dir_all(xcframework.join("ios-arm64-simulator").join(headers_path))?;

    fs::write(
        xcframework.join("Info.plist"),
        info_plist_content(&opts.lib_name, &headers_path),
    )?;

    for target in targets {
        let lib = format!("lib{}.a", opts.lib_name);
        let from = target_dir.join(&target).join("release").join(&lib);

        if target.contains("sim") {
            fs::copy(from, xcframework.join("ios-arm64-simulator").join(&lib))?;
            debug!("Copied {} to ios-arm64-simulator", &lib);
        } else {
            fs::copy(from, xcframework.join("ios-arm64").join(&lib))?;
            debug!("Copied {} to ios-arm64", &lib);
        }
    }

    Ok(())
}

fn info_plist_content(lib_name: &str, headers_path: &str) -> String {
    let lib_value = format!("      <string>lib{}.a</string>", lib_name);
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
