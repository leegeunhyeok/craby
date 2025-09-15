pub mod template {
    use craby_common::{
        constants::dest_lib_name,
        utils::string::{flat_case, SanitizedString},
    };
    use indoc::formatdoc;

    use crate::constants::{cxx_mod_cls_name, objc_mod_provider_name};

    pub fn objc_mod_provider(name: &String) -> String {
        let cxx_mod_cls_name = cxx_mod_cls_name(name);
        let objc_mod_provider_name = objc_mod_provider_name(name);
        let flat_name = flat_case(name);

        formatdoc! {
            r#"
            #import "{cxx_mod_cls_name}.hpp"
            #import <ReactCommon/CxxTurboModuleUtils.h>

            @interface {objc_mod_provider_name} : NSObject
            @end

            @implementation {objc_mod_provider_name}
            + (void)load {{
              facebook::react::registerCxxModuleToGlobalModuleMap(
                  craby::{flat_name}::{cxx_mod_cls_name}::kModuleName,
                  [](std::shared_ptr<facebook::react::CallInvoker> jsInvoker) {{
                  return std::make_shared<craby::{flat_name}::{cxx_mod_cls_name}>(jsInvoker);
                  }});
            }}
            @end"#,
          cxx_mod_cls_name = cxx_mod_cls_name,
          objc_mod_provider_name = objc_mod_provider_name,
          flat_name = flat_name,
        }
    }

    pub fn info_plist(name: &String, lib_identifier: &str, lib_sim_identifier: &str) -> String {
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
            lib_identifier = lib_identifier,
            lib_sim_identifier = lib_sim_identifier,
        }
    }
}
