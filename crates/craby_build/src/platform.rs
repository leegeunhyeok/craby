pub mod android {
    use std::{fs, path::PathBuf};

    use craby_common::{
        constants::{self, binding_header_dir, crate_target_dir, lib_header_name, lib_name},
        utils::string::SanitizedString,
    };
    use log::debug;

    use crate::{
        constants::android::{ABI_ARM64_V8A, ABI_ARMEABI_V7A, ABI_X86, ABI_X86_64, INCLUDE_DIR},
        utils::android::get_abi_by_target,
    };

    pub struct CreateAbiFilesOptions {
        pub project_root: PathBuf,
        pub header_path: PathBuf,
        pub lib_name: SanitizedString,
    }

    pub fn create_abi_files(opts: CreateAbiFilesOptions) -> Result<(), anyhow::Error> {
        let lib_name = lib_name(&opts.lib_name);
        let lib_header = lib_header_name(&opts.lib_name);
        let abi_base_path = abi_base_path(&opts.project_root);

        prepare_abi_dirs(&abi_base_path)?;

        for target in get_targets() {
            let abi = get_abi_by_target(&target);

            let from_lib = crate_target_dir(&opts.project_root, &target).join(&lib_name);
            let from_header = binding_header_dir(&opts.project_root).join(&lib_header);

            let to = abi_base_path.join(abi);
            let to_lib = to.join(&lib_name);
            let to_header = to.join(INCLUDE_DIR).join(&lib_header);

            debug!(
                "(Library) Copying {} to {}",
                from_lib.display(),
                to_lib.display()
            );
            debug!(
                "(Headers) Copying {} to {}",
                from_header.display(),
                to_header.display()
            );
            fs::copy(from_lib, to_lib)?;
            fs::copy(from_header, to_header)?;
        }

        Ok(())
    }

    fn prepare_abi_dirs(abi_base_path: &PathBuf) -> Result<(), anyhow::Error> {
        if abi_base_path.exists() {
            fs::remove_dir_all(&abi_base_path)?;
            debug!("Cleaned up existing abi base directory");
        }

        for abi in [ABI_ARM64_V8A, ABI_ARMEABI_V7A, ABI_X86_64, ABI_X86] {
            fs::create_dir_all(abi_base_path.join(abi).join(INCLUDE_DIR))?;
        }

        Ok(())
    }

    fn get_targets() -> impl Iterator<Item = String> {
        constants::toolchain::TARGETS.iter().filter_map(|target| {
            if target.contains("android") {
                Some(target.to_string())
            } else {
                None
            }
        })
    }

    fn abi_base_path(project_root: &PathBuf) -> PathBuf {
        project_root.join("android").join("src").join("libs")
    }
}

pub mod ios {
    use std::{fs, path::PathBuf};

    use indoc::formatdoc;
    use log::debug;

    use crate::constants::{self, ios::{HEADERS_DIR, LIB_IDENTIFIER_ARM64, LIB_IDENTIFIER_ARM64_SIMULATOR}};
    use craby_common::{
        constants::{binding_header_dir, crate_target_dir, lib_header_name, lib_name},
        utils::{ios::xcframework_name, string::SanitizedString},
    };

    pub struct CreateXcframeworkOptions {
        pub project_root: PathBuf,
        pub header_path: PathBuf,
        pub lib_name: SanitizedString,
    }

    pub fn create_xcframework(opts: CreateXcframeworkOptions) -> Result<(), anyhow::Error> {
        let lib_name = lib_name(&opts.lib_name);
        let lib_header = lib_header_name(&opts.lib_name);
        let xcframework_path = ios_framework_path(&opts.project_root, &opts.lib_name);

        prepare_xcframework(&xcframework_path)?;

        fs::write(
            xcframework_path.join("Info.plist"),
            info_plist_content(&lib_name),
        )?;
        debug!("Wrote Info.plist");

        for target in get_targets() {
            let from_lib = crate_target_dir(&opts.project_root, &target).join(&lib_name);
            let from_header = binding_header_dir(&opts.project_root).join(&lib_header);

            let to = xcframework_path.join(get_lib_identifier(&target));
            let to_lib = to.join(&lib_name);
            let to_header = to.join(HEADERS_DIR).join(&lib_header);

            debug!(
                "(Library) Copying {} to {}",
                from_lib.display(),
                to_lib.display()
            );
            debug!(
                "(Headers) Copying {} to {}",
                from_header.display(),
                to_header.display()
            );
            fs::copy(from_lib, to_lib)?;
            fs::copy(from_header, to_header)?;
        }

        Ok(())
    }

    fn prepare_xcframework(xcframework: &PathBuf) -> Result<(), anyhow::Error> {
        if xcframework.exists() {
            fs::remove_dir_all(&xcframework)?;
        }

        fs::create_dir_all(xcframework.join("ios-arm64").join(HEADERS_DIR))?;
        fs::create_dir_all(xcframework.join("ios-arm64-simulator").join(HEADERS_DIR))?;

        Ok(())
    }

    fn get_lib_identifier(target: &String) -> String {
        if target.contains("sim") {
            "ios-arm64-simulator"
        } else {
            "ios-arm64"
        }
        .to_string()
    }

    fn get_targets() -> impl Iterator<Item = String> {
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
                        <key>LibraryIdentifier</key>
                        <string>{lib_identifier}</string>
                        <key>LibraryPath</key>
                        <string>{lib_name}</string>
                        <key>HeadersPath</key>
                        <string>{headers_path}</string>
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
                        <key>HeadersPath</key>
                        <string>{headers_path}</string>
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
            lib_identifier = LIB_IDENTIFIER_ARM64,
            lib_sim_identifier = LIB_IDENTIFIER_ARM64_SIMULATOR,
            headers_path = HEADERS_DIR
        }
    }

    fn ios_framework_path(project_root: &PathBuf, lib_name: &SanitizedString) -> PathBuf {
        project_root
            .join("ios")
            .join("framework")
            .join(xcframework_name(lib_name))
    }
}
