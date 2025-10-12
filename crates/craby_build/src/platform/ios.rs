use std::{fs, path::PathBuf, process::Command};

use crate::{
    cargo::artifact::{ArtifactType, Artifacts},
    constants::{ios::Identifier, toolchain::Target},
    cxx::replace_cxx_header,
};

use craby_common::{
    config::CompleteCrabyConfig,
    constants::{crate_target_dir, dest_lib_name, ios_base_path, lib_base_name},
    utils::string::SanitizedString,
};
use indoc::formatdoc;
use log::debug;

const IOS_TARGETS: [Target; 3] = [
    Target::Ios(Identifier::Arm64),
    Target::Ios(Identifier::Arm64Simulator),
    Target::Ios(Identifier::X86_64Simulator),
];

pub fn crate_libs<'a>(config: &'a CompleteCrabyConfig) -> Result<(), anyhow::Error> {
    let ios_base_path = ios_base_path(&config.project_root);

    let (sims, devices): (Vec<_>, Vec<_>) = IOS_TARGETS.iter().partition(|target| match target {
        Target::Ios(Identifier::Arm64Simulator) | Target::Ios(Identifier::X86_64Simulator) => true,
        _ => false,
    });

    let sims = sims
        .into_iter()
        .map(|target| Artifacts::get_artifacts(config, target))
        .collect::<Result<Vec<_>, anyhow::Error>>()?;

    let devices = devices
        .into_iter()
        .map(|target| Artifacts::get_artifacts(config, target))
        .collect::<Result<Vec<_>, anyhow::Error>>()?;

    let sims = create_sim_lib(&config.project_root, sims)?;
    let xcframework_path = create_xcframework(&config)?;

    for artifacts in [devices, vec![sims]].concat() {
        // ios/src
        artifacts.copy_to(ArtifactType::Src, &ios_base_path.join("src"))?;

        // ios/include
        artifacts.copy_to(ArtifactType::Header, &ios_base_path.join("include"))?;

        // ios/framework/lib{lib_name}.xcframework/{identifier}
        artifacts.copy_to(
            ArtifactType::Lib,
            &xcframework_path.join(&artifacts.identifier),
        )?;
    }

    let signal_path = ios_base_path.join("include").join("signals.h");
    if signal_path.try_exists()? {
        replace_cxx_header(&signal_path)?;
    }

    Ok(())
}

/// Creates a simulator library from the given artifacts
///
/// This function takes a vector of artifacts and creates a simulator library from them.
/// It uses the `lipo` command to combine the libraries into a single library.
fn create_sim_lib(
    project_root: &PathBuf,
    sims: Vec<Artifacts>,
) -> Result<Artifacts, anyhow::Error> {
    let identifier = Identifier::Simulator.try_into_str()?;
    let orig = sims
        .first()
        .cloned()
        .ok_or(anyhow::anyhow!("No simulator artifacts found"))?;

    let libs = sims
        .into_iter()
        .map(|artifacts| artifacts.libs)
        .flatten()
        .collect::<Vec<_>>();

    let lib = libs.get(0).ok_or(anyhow::anyhow!("No library found"))?;
    let lib_name = lib
        .file_name()
        .ok_or(anyhow::anyhow!("No library name found"))?;

    let dest_dir = crate_target_dir(project_root, identifier);
    let dest_path = dest_dir.join(lib_name);

    if dest_dir.try_exists()? {
        fs::remove_dir_all(&dest_dir)?;
    }
    fs::create_dir_all(&dest_dir)?;

    debug!(
        "Creating simulator library from artifacts (dest: {:?})",
        dest_path
    );

    let res = Command::new("lipo")
        .arg("-create")
        .args(libs)
        .args(["-output", dest_path.to_str().unwrap()])
        .output()?;

    if !res.status.success() {
        anyhow::bail!(
            "Failed to create simulator library: {}",
            String::from_utf8_lossy(&res.stderr)
        );
    }

    Ok(Artifacts {
        identifier: Identifier::Simulator.try_into_str()?.to_string(),
        headers: orig.headers,
        srcs: orig.srcs,
        libs: vec![dest_path],
    })
}

fn create_xcframework(config: &CompleteCrabyConfig) -> Result<PathBuf, anyhow::Error> {
    let name = SanitizedString::from(&config.project.name);
    let lib_base_name = lib_base_name(&name);
    let info_plist_content = info_plist(&config.project.name)?;
    let framework_path = ios_base_path(&config.project_root).join("framework");
    let xcframework_path =
        framework_path.join(format!("lib{}.xcframework", lib_base_name.to_string()));

    if xcframework_path.try_exists()? {
        fs::remove_dir_all(&xcframework_path)?;
    }

    fs::create_dir_all(&xcframework_path)?;

    let info_plist_path = xcframework_path.join("Info.plist");
    fs::write(info_plist_path, info_plist_content)?;

    Ok(xcframework_path)
}

pub fn info_plist(name: &String) -> Result<String, anyhow::Error> {
    let lib_name = dest_lib_name(&SanitizedString::from(name));

    let content = formatdoc! {
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
                        <string>x86_64</string>
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
        lib_identifier = Identifier::Arm64.try_into_str()?,
        lib_sim_identifier = Identifier::Simulator.try_into_str()?,
    };

    Ok(content)
}
