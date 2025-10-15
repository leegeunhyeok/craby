use std::path::PathBuf;

use craby_common::{
    constants::{android_path, dest_lib_name, jni_base_path},
    utils::string::{flat_case, kebab_case, SanitizedString},
};
use indoc::formatdoc;

use crate::{
    constants::cxx_mod_cls_name,
    types::{CodegenContext, Schema},
    utils::indent_str,
};

use super::types::{GenerateResult, Generator, GeneratorInvoker, Template};

pub struct AndroidTemplate;
pub struct AndroidGenerator;

pub enum AndroidFileType {
    JNIEntry,
    CmakeLists,
}

impl AndroidTemplate {
    fn file_path(&self, file_type: &AndroidFileType) -> PathBuf {
        match file_type {
            AndroidFileType::JNIEntry => PathBuf::from("OnLoad.cpp"),
            AndroidFileType::CmakeLists => PathBuf::from("CMakeLists.txt"),
        }
    }

    /// Returns `JNI_OnLoad` function implementation
    ///
    /// # Generated Code
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
    fn jni_entry(&self, schemas: &Vec<Schema>) -> Result<String, anyhow::Error> {
        let mut cxx_includes = vec![];
        let mut cxx_registers = vec![];

        for schema in schemas {
            let cxx_mod = cxx_mod_cls_name(&schema.module_name);
            let flat_name = flat_case(&schema.module_name);

            let cxx_namespace = format!("craby::{}::{}", flat_name, cxx_mod);
            let cxx_include = format!("#include <{cxx_mod}.hpp>");
            let cxx_register = formatdoc! {
                r#"
                facebook::react::registerCxxModuleToGlobalModuleMap(
                  {cxx_namespace}::kModuleName,
                  [](std::shared_ptr<facebook::react::CallInvoker> jsInvoker) {{
                    return std::make_shared<{cxx_namespace}>(jsInvoker);
                  }});"#,
                cxx_namespace = cxx_namespace
            };

            cxx_includes.push(cxx_include);
            cxx_registers.push(cxx_register);
        }

        let content = formatdoc! {
            r#"
            {cxx_includes}

            #include <jni.h>
            #include <ReactCommon/CxxTurboModuleUtils.h>

            jint JNI_OnLoad(JavaVM *vm, void *reserved) {{
            {cxx_registers}
              return JNI_VERSION_1_6;
            }}"#,
            cxx_includes = cxx_includes.join("\n"),
            cxx_registers = indent_str(cxx_registers.join("\n"), 2),
        };

        Ok(content)
    }

    /// Generates the CMakeLists.txt for Android native module build configuration.
    ///
    /// # Generated Code
    ///
    /// ```cmake
    /// cmake_minimum_required(VERSION 3.13)
    ///
    /// project(craby-my-app)
    ///
    /// set (CMAKE_VERBOSE_MAKEFILE ON)
    /// set (CMAKE_CXX_STANDARD 20)
    ///
    /// find_package(ReactAndroid REQUIRED CONFIG)
    ///
    /// # Import the pre-built Craby library
    /// add_library(my-app-lib STATIC IMPORTED)
    /// set_target_properties(my-app-lib PROPERTIES
    ///   IMPORTED_LOCATION "${CMAKE_SOURCE_DIR}/src/main/jni/libs/${ANDROID_ABI}/libcraby_my_app.a"
    /// )
    /// target_include_directories(my-app-lib INTERFACE
    ///   "${CMAKE_SOURCE_DIR}/src/main/jni/include"
    /// )
    ///
    /// # Generated C++ source files by Craby
    /// add_library(cxx-my-app SHARED
    ///   src/main/jni/OnLoad.cpp
    ///   src/main/jni/src/ffi.rs.cc
    ///   ../cpp/CxxMyTestModule.cpp
    /// )
    /// target_include_directories(cxx-my-app PRIVATE
    ///   ../cpp
    /// )
    ///
    /// target_link_libraries(cxx-my-app
    ///   # android
    ///   ReactAndroid::reactnative
    ///   ReactAndroid::jsi
    ///   # my-app-lib
    ///   my-app-lib
    /// )
    ///
    /// # From ReactAndroid/cmake-utils/folly-flags.cmake
    /// target_compile_definitions(cxx-craby-test PRIVATE
    ///   -DFOLLY_NO_CONFIG=1
    ///   -DFOLLY_HAVE_CLOCK_GETTIME=1
    ///   -DFOLLY_USE_LIBCPP=1
    ///   -DFOLLY_CFG_NO_COROUTINES=1
    ///   -DFOLLY_MOBILE=1
    ///   -DFOLLY_HAVE_RECVMMSG=1
    ///   -DFOLLY_HAVE_PTHREAD=1
    ///   # Once we target android-23 above, we can comment
    ///   # the following line. NDK uses GNU style stderror_r() after API 23.
    ///   -DFOLLY_HAVE_XSI_STRERROR_R=1
    /// )
    /// ```
    fn cmakelists(&self, project: &CodegenContext) -> String {
        let kebab_name = kebab_case(&project.name);
        let lib_name = dest_lib_name(&SanitizedString::from(&project.name));
        let cxx_mod_cpp_files = project
            .schemas
            .iter()
            .map(|schema| format!("../cpp/{}.cpp", cxx_mod_cls_name(&schema.module_name)))
            .collect::<Vec<_>>();

        formatdoc! {
            r#"
            cmake_minimum_required(VERSION 3.13)

            project(craby-{kebab_name})

            set (CMAKE_VERBOSE_MAKEFILE ON)
            set (CMAKE_CXX_STANDARD 20)

            find_package(ReactAndroid REQUIRED CONFIG)

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
            {cxx_mod_cpp_files}
            )
            target_include_directories(cxx-{kebab_name} PRIVATE
              ../cpp
            )

            target_link_libraries(cxx-{kebab_name}
              # android
              ReactAndroid::reactnative
              ReactAndroid::jsi
              # {kebab_name}-lib
              {kebab_name}-lib
            )

            # From ReactAndroid/cmake-utils/folly-flags.cmake
            target_compile_definitions(cxx-craby-test PRIVATE
              -DFOLLY_NO_CONFIG=1
              -DFOLLY_HAVE_CLOCK_GETTIME=1
              -DFOLLY_USE_LIBCPP=1
              -DFOLLY_CFG_NO_COROUTINES=1
              -DFOLLY_MOBILE=1
              -DFOLLY_HAVE_RECVMMSG=1
              -DFOLLY_HAVE_PTHREAD=1
              # Once we target android-23 above, we can comment
              # the following line. NDK uses GNU style stderror_r() after API 23.
              -DFOLLY_HAVE_XSI_STRERROR_R=1
            )"#,
            kebab_name = kebab_name,
            lib_name = lib_name,
            cxx_mod_cpp_files = indent_str(cxx_mod_cpp_files.join("\n"), 2),
        }
    }
}

impl Template for AndroidTemplate {
    type FileType = AndroidFileType;

    fn render(
        &self,
        project: &CodegenContext,
        file_type: &Self::FileType,
    ) -> Result<Vec<(PathBuf, String)>, anyhow::Error> {
        let path = self.file_path(file_type);
        let content = match file_type {
            AndroidFileType::JNIEntry => self.jni_entry(&project.schemas),
            AndroidFileType::CmakeLists => Ok(self.cmakelists(project)),
        }?;

        Ok(vec![(path, content)])
    }
}

impl Default for AndroidGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl AndroidGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl Generator<AndroidTemplate> for AndroidGenerator {
    fn cleanup(_: &CodegenContext) -> Result<(), anyhow::Error> {
        Ok(())
    }

    fn generate(&self, project: &CodegenContext) -> Result<Vec<GenerateResult>, anyhow::Error> {
        let android_base_path = android_path(&project.root);
        let jni_base_path = jni_base_path(&project.root);
        let template = self.template_ref();
        let mut files = vec![];

        let jni_res = template
            .render(project, &AndroidFileType::JNIEntry)?
            .into_iter()
            .map(|(path, content)| GenerateResult {
                path: jni_base_path.join(path),
                content,
                overwrite: true,
            })
            .collect::<Vec<_>>();

        let cmake_res = template
            .render(project, &AndroidFileType::CmakeLists)?
            .into_iter()
            .map(|(path, content)| GenerateResult {
                path: android_base_path.join(path),
                content,
                overwrite: true,
            })
            .collect::<Vec<_>>();

        files.extend(jni_res);
        files.extend(cmake_res);

        Ok(files)
    }

    fn template_ref(&self) -> &AndroidTemplate {
        &AndroidTemplate
    }
}

impl GeneratorInvoker for AndroidGenerator {
    fn invoke_generate(
        &self,
        project: &CodegenContext,
    ) -> Result<Vec<GenerateResult>, anyhow::Error> {
        self.generate(project)
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use crate::tests::get_codegen_context;

    use super::*;

    #[test]
    fn test_android_generator() {
        let ctx = get_codegen_context();
        let generator = AndroidGenerator::new();
        let results = generator.generate(&ctx).unwrap();
        let result = results
            .iter()
            .map(|res| format!("{}\n{}", res.path.display(), res.content))
            .collect::<Vec<_>>()
            .join("\n\n");

        assert_snapshot!(result);
    }
}
