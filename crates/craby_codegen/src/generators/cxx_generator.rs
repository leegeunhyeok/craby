use std::path::PathBuf;

use craby_common::{constants::cxx_dir, utils::string::flat_case};
use indoc::formatdoc;

use crate::{
    constants::cxx_mod_cls_name, platform::cxx::CxxMethod, types::schema::Schema, utils::indent_str,
};

use super::types::{GenerateResult, Generator, GeneratorInvoker, Template};

pub struct CxxTemplate;
pub struct CxxGenerator;

pub enum CxxFileType {
    /// cpp/hpp files
    Mod,
    /// bridging-generated.hpp
    BridgingHpp,
}

impl CxxTemplate {
    fn cxx_methods(&self, schema: &Schema) -> Result<Vec<CxxMethod>, anyhow::Error> {
        let res = schema
            .spec
            .methods
            .iter()
            .map(|spec| spec.as_cxx_method(&schema.module_name))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(res)
    }

    /// Returns the cxx JSI method definition.
    ///
    /// ```cpp
    /// static facebook::jsi::Value
    /// myFunc(facebook::jsi::Runtime &rt,
    ///        facebook::react::TurboModule &turboModule,
    ///        const facebook::jsi::Value args[], size_t count);
    /// ```
    fn cxx_method_def(&self, name: &String) -> String {
        formatdoc! {
            r#"
            static facebook::jsi::Value
            {name}(facebook::jsi::Runtime &rt,
                facebook::react::TurboModule &turboModule,
                const facebook::jsi::Value args[], size_t count);"#,
            name = name,
        }
    }

    /// Returns the complete cxx TurboModule source/header files.
    fn cxx_mod(&self, schema: &Schema) -> Result<(String, String), anyhow::Error> {
        let flat_name = flat_case(&schema.module_name);
        let cxx_mod = cxx_mod_cls_name(&schema.module_name);
        let cxx_methods = self.cxx_methods(schema)?;
        let include_stmt = format!("#include \"{}.hpp\"", cxx_mod);

        // Assign method metadata with function pointer to the TurboModule's method map
        //
        // ```cpp
        // methodMap_["multiply"] = MethodMetadata{1, &CxxMyTestModule::multiply};
        // ```
        let method_maps = cxx_methods
            .iter()
            .map(|method| format!("methodMap_[\"{}\"] = {};", method.name, method.metadata))
            .collect::<Vec<_>>()
            .join("\n");

        let method_defs = cxx_methods
            .iter()
            .map(|method| self.cxx_method_def(&method.name))
            .collect::<Vec<_>>()
            .join("\n\n");

        // Functions implementations
        //
        // ```cpp
        // jsi::Value CxxMyTestModule::multiply(jsi::Runtime &rt,
        //                                    react::TurboModule &turboModule,
        //                                    const jsi::Value args[],
        //                                    size_t count) {
        //     // ...
        // }
        // ```
        let method_impls = cxx_methods
            .into_iter()
            .map(|method| method.impl_func)
            .collect::<Vec<_>>()
            .join("\n\n");

        // ```cpp
        // namespace mymodule {
        //
        // CxxMyTestModule::CxxMyTestModule(
        //     std::shared_ptr<react::CallInvoker> jsInvoker)
        //     : TurboModule(CxxMyTestModule::kModuleName, jsInvoker) {
        //   callInvoker_ = std::move(jsInvoker);
        //
        //   // Method maps
        // }
        //
        // /* Method implementations */
        //
        // } // namespace mymodule
        // ```
        let cpp = formatdoc! {
            r#"
            namespace {flat_name} {{

            {cxx_mod}::{cxx_mod}(
                std::shared_ptr<react::CallInvoker> jsInvoker)
                : TurboModule({cxx_mod}::kModuleName, jsInvoker) {{
              callInvoker_ = std::move(jsInvoker);
            
            {method_maps}
            }}
            
            {method_impls}
            
            }} // namespace {flat_name}"#,
            flat_name = flat_name,
            cxx_mod = cxx_mod,
            method_maps = indent_str(method_maps, 2),
            method_impls = method_impls,
        };

        let hpp = formatdoc! {
            r#"
            namespace {flat_name} {{

            class JSI_EXPORT {cxx_mod} : public facebook::react::TurboModule {{
            public:
              static constexpr const char *kModuleName = "{turbo_module_name}";

              {cxx_mod}(std::shared_ptr<facebook::react::CallInvoker> jsInvoker);

            {method_defs}

            protected:
              std::shared_ptr<facebook::react::CallInvoker> callInvoker_;
            }};

            }} // namespace {flat_name}"#,
            flat_name = flat_name,
            cxx_mod = cxx_mod,
            turbo_module_name = schema.module_name,
            method_defs = indent_str(method_defs, 2),
        };

        // ```cpp
        // #include "my_module.hpp"
        //
        // #include <thread>
        // #include <react/bridging/Bridging.h>
        //
        // #include "cxx.h"
        // #include "ffi.rs.h"
        // #include "bridging-generated.hpp"
        // #include "utils.hpp"
        //
        // using namespace facebook;
        //
        // namespace craby {
        // // TurboModule implementations
        // } // namespace craby
        // ```
        let cpp_content = formatdoc! {
            r#"
            {include_stmt}

            #include <thread>
            #include <react/bridging/Bridging.h>

            #include "cxx.h"
            #include "ffi.rs.h"
            #include "bridging-generated.hpp"
            #include "utils.hpp"

            using namespace facebook;

            namespace craby {{
            {cpp}
            }} // namespace craby"#,
            include_stmt = include_stmt,
            cpp = cpp,
        };

        let hpp_content = formatdoc! {
            r#"
            #pragma once

            #include <memory>
            #include <ReactCommon/TurboModule.h>
            #include <jsi/jsi.h>

            namespace craby {{
            {hpp}
            }} // namespace craby"#,
            hpp = hpp,
        };

        Ok((cpp_content, hpp_content))
    }

    fn cxx_bridging(&self, schemas: &Vec<Schema>) -> Result<String, anyhow::Error> {
        let bridging_templates = schemas
            .iter()
            .flat_map(|schema| schema.as_cxx_bridging_templates())
            .flatten()
            .collect::<Vec<_>>();

        let cxx_bridging = formatdoc! {
            r#"
            #pragma once

            #include <react/bridging/Bridging.h>
            #include "cxx.h"
            #include "ffi.rs.h"

            using namespace facebook;

            namespace facebook {{
            namespace react {{

            template <>
            struct Bridging<rust::String> {{
              static rust::String fromJs(jsi::Runtime& rt, const jsi::Value &value, std::shared_ptr<CallInvoker> callInvoker) {{
                auto str = value.asString(rt).utf8(rt);
                return rust::String(str);
              }}

              static jsi::Value toJs(jsi::Runtime& rt, const rust::String& value) {{
                return react::bridging::toJs(rt, std::string(value));
              }}
            }};

            template <typename T>
            struct Bridging<rust::Vec<T>> {{
              static rust::Vec<T> fromJs(jsi::Runtime& rt, const jsi::Value &value, std::shared_ptr<CallInvoker> callInvoker) {{
                auto arr = value.asObject(rt).asArray(rt);
                size_t len = arr.length(rt);
                rust::Vec<T> vec;
                vec.reserve(len);

                for (size_t i = 0; i < len; i++) {{
                  auto element = arr.getValueAtIndex(rt, i);
                  vec.push_back(react::bridging::fromJs<T>(rt, element, callInvoker));
                }}

                return vec;
              }}

              static jsi::Array toJs(jsi::Runtime& rt, const rust::Vec<T>& vec) {{
                auto arr = jsi::Array(rt, vec.size());

                for (size_t i = 0; i < vec.size(); i++) {{
                  auto jsElement = react::bridging::toJs(rt, vec[i]);
                  arr.setValueAtIndex(rt, i, jsElement);
                }}

                return arr;
              }}
            }};
            {bridging_templates}
            }} // namespace react
            }} // namespace facebook"#,
            bridging_templates = if bridging_templates.is_empty() { "".to_string() } else { format!("\n{}\n", bridging_templates.join("\n\n")) },
        };

        Ok(cxx_bridging)
    }
}

impl Template for CxxTemplate {
    type FileType = CxxFileType;

    fn render(
        &self,
        schemas: &Vec<Schema>,
        file_type: &Self::FileType,
    ) -> Result<Vec<(PathBuf, String)>, anyhow::Error> {
        let res = match file_type {
            CxxFileType::Mod => schemas
                .iter()
                .flat_map(|schema| -> Result<Vec<(PathBuf, String)>, anyhow::Error> {
                    let (cpp, hpp) = self.cxx_mod(schema)?;
                    let cxx_mod = cxx_mod_cls_name(&schema.module_name);
                    let files = vec![
                        (PathBuf::from(format!("{}.cpp", cxx_mod)), cpp),
                        (PathBuf::from(format!("{}.hpp", cxx_mod)), hpp),
                    ];
                    Ok(files)
                })
                .flatten()
                .collect::<Vec<_>>(),
            CxxFileType::BridgingHpp => vec![(
                PathBuf::from("bridging-generated.hpp"),
                self.cxx_bridging(schemas)?,
            )],
        };

        Ok(res)
    }
}

impl CxxGenerator {
    pub fn new() -> Self {
        Self {}
    }
}

impl Generator<CxxTemplate> for CxxGenerator {
    fn generate(
        &self,
        project_root: &PathBuf,
        schemas: &Vec<Schema>,
    ) -> Result<Vec<GenerateResult>, anyhow::Error> {
        let base_path = cxx_dir(project_root);
        let template = self.template_ref();
        let res = [
            template.render(schemas, &CxxFileType::Mod)?,
            template.render(schemas, &CxxFileType::BridgingHpp)?,
        ]
        .into_iter()
        .flatten()
        .map(|(path, content)| GenerateResult {
            path: base_path.join(path),
            content,
            overwrite: true,
        })
        .collect::<Vec<_>>();

        Ok(res)
    }

    fn template_ref(&self) -> &CxxTemplate {
        &CxxTemplate
    }
}

impl GeneratorInvoker for CxxGenerator {
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
    fn test_cxx_generator() {
        let schema = load_schema_json::<Schema>();
        let generator = CxxGenerator::new();
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
