use std::path::PathBuf;

use craby_common::{
    constants::{android_path, dest_lib_name, jni_base_path},
    utils::string::{flat_case, kebab_case, SanitizedString},
};
use indoc::formatdoc;

use crate::{constants::cxx_mod_cls_name, types::schema::Schema, utils::indent_str};

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

        schemas.iter().for_each(|schema| {
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
                  }});
              "#,
              cxx_namespace = cxx_namespace
            };

            cxx_includes.push(cxx_include);
            cxx_registers.push(cxx_register);
        });

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

    fn cmakelists(&self, project_name: &String) -> String {
        let kebab_name = kebab_case(project_name);
        let lib_name = dest_lib_name(&SanitizedString::from(project_name));
        let cxx_mod_cls_name = cxx_mod_cls_name(project_name);

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
            )"#,
            kebab_name = kebab_name,
            lib_name = lib_name,
            cxx_mod_cls_name = cxx_mod_cls_name,
        }
    }
}

impl Template for AndroidTemplate {
    type FileType = AndroidFileType;

    fn render(
        &self,
        schemas: &Vec<Schema>,
        file_type: &Self::FileType,
    ) -> Result<Vec<(PathBuf, String)>, anyhow::Error> {
        let path = self.file_path(file_type);
        let content = match file_type {
            AndroidFileType::JNIEntry => self.jni_entry(schemas),
            AndroidFileType::CmakeLists => {
                // TODO: Support multiple schemas
                Ok(self.cmakelists(&schemas.get(0).unwrap().module_name))
            }
        }?;

        Ok(vec![(path, content)])
    }
}

impl AndroidGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl Generator<AndroidTemplate> for AndroidGenerator {
    fn generate(
        &self,
        project_root: &PathBuf,
        schemas: &Vec<Schema>,
    ) -> Result<Vec<GenerateResult>, anyhow::Error> {
        let android_base_path = android_path(project_root);
        let jni_base_path = jni_base_path(project_root);
        let template = self.template_ref();
        let mut files = vec![];

        let jni_res = template
            .render(schemas, &AndroidFileType::JNIEntry)?
            .into_iter()
            .map(|(path, content)| GenerateResult {
                path: jni_base_path.join(path),
                content,
                overwrite: true,
            })
            .collect::<Vec<_>>();

        let cmake_res = template
            .render(schemas, &AndroidFileType::CmakeLists)?
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
        project_root: &PathBuf,
        schemas: &Vec<Schema>,
    ) -> Result<Vec<GenerateResult>, anyhow::Error> {
        self.generate(project_root, schemas)
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use crate::tests::load_schema_json;

    use super::*;

    #[test]
    fn test_android_generator() {
        let schema = load_schema_json::<Schema>();
        let generator = AndroidGenerator::new();
        let results = generator
            .generate(&PathBuf::from("."), &vec![schema])
            .unwrap();

        assert_snapshot!(results
            .iter()
            .map(|res| format!("{}\n{}", res.path.display(), res.content))
            .collect::<Vec<_>>()
            .join("\n\n"));
    }
}
