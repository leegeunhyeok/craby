pub mod template {
    use craby_common::{
        constants::dest_lib_name,
        utils::string::{flat_case, kebab_case, SanitizedString},
    };
    use indoc::formatdoc;

    use crate::constants::cxx_mod_cls_name;

    /// Returns `JNI_OnLoad` function implementation
    ///
    /// ```cpp
    /// jint JNI_OnLoad(JavaVM *vm, void *reserved) {
    ///   facebook::react::registerCxxModuleToGlobalModuleMap(
    ///     craby::mymodule::MyTestModule::kModuleName,
    ///     [](std::shared_ptr<facebook::react::CallInvoker> jsInvoker) {
    ///       return std::make_shared<craby::mymodule::MyTestModule>(jsInvoker);
    ///     });
    ///   return JNI_VERSION_1_6;
    /// }
    /// ```
    pub fn cxx_on_load(name: &String) -> String {
        let cxx_mod = cxx_mod_cls_name(name);
        let flat_name = flat_case(name);

        formatdoc! {
          r#"
          #include <jni.h>
          #include <ReactCommon/CxxTurboModuleUtils.h>
          #include <{cxx_mod}.hpp>

          jint JNI_OnLoad(JavaVM *vm, void *reserved) {{
            facebook::react::registerCxxModuleToGlobalModuleMap(
                craby::{flat_name}::{cxx_mod}::kModuleName,
                [](std::shared_ptr<facebook::react::CallInvoker> jsInvoker) {{
                  return std::make_shared<craby::{flat_name}::{cxx_mod}>(jsInvoker);
                }});
            return JNI_VERSION_1_6;
          }}
          "#,
          cxx_mod = cxx_mod,
          flat_name = flat_name,
        }
    }

    /// Returns `CMakeLists.txt`
    pub fn cmakelists(name: &String) -> String {
        let kebab_name = kebab_case(name);
        let lib_name = dest_lib_name(&SanitizedString::from(name));
        let cxx_mod_cls_name = cxx_mod_cls_name(name);

        formatdoc! {
          r#"
          cmake_minimum_required(VERSION 3.13)

          project(craby-{kebab_name})

          set (CMAKE_VERBOSE_MAKEFILE ON)
          set (CMAKE_CXX_STANDARD 20)

          find_package(ReactAndroid REQUIRED CONFIG)
          find_package(hermes-engine REQUIRED CONFIG)

          # Import the pre-built Craby library
          add_library({kebab_name}-lib STATIC IMPORTED)
          set_target_properties({kebab_name}-lib PROPERTIES
            IMPORTED_LOCATION "${{CMAKE_SOURCE_DIR}}/src/main/jni/libs/${{ANDROID_ABI}}/{lib_name}"
          )
          target_include_directories({kebab_name}-lib INTERFACE
            "${{CMAKE_SOURCE_DIR}}/src/main/jni/include"
          )

          # Generated C++ source files by Craby
          add_library(cxx-{kebab_name} SHARED
            src/main/jni/OnLoad.cpp
            src/main/jni/src/ffi.rs.cc
            ../cpp/{cxx_mod_cls_name}.cpp
          )
          target_include_directories(cxx-{kebab_name} PRIVATE
            ../cpp
          )

          target_link_libraries(cxx-{kebab_name}
            # android
            ReactAndroid::reactnative
            ReactAndroid::jsi
            hermes-engine::libhermes
            # {kebab_name}-lib
            {kebab_name}-lib
          )
          "#,
          kebab_name = kebab_name,
          lib_name = lib_name,
          cxx_mod_cls_name = cxx_mod_cls_name,
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::*;

    const MODULE_NAME: &str = "CrabyTest";

    #[test]
    fn test_cxx_on_load() {
        let result = template::cxx_on_load(&MODULE_NAME.to_string());

        assert_snapshot!(result);
    }

    #[test]
    fn test_cmakelists() {
        let result = template::cmakelists(&MODULE_NAME.to_string());

        assert_snapshot!(result);
    }
}
