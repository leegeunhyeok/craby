use std::path::PathBuf;

use craby_common::{
    constants::{cxx_bridge_include_dir, cxx_dir},
    utils::string::flat_case,
};
use indoc::formatdoc;

use crate::{
    constants::cxx_mod_cls_name,
    platform::cxx::CxxMethod,
    types::{CodegenContext, Schema},
    utils::indent_str,
};

use super::types::{GenerateResult, Generator, GeneratorInvoker, Template};

pub struct CxxTemplate;
pub struct CxxGenerator;

pub enum CxxFileType {
    /// cpp/hpp files
    Mod,
    /// bridging-generated.hpp
    BridgingHpp,
    /// signals.h
    SignalsH,
}

impl CxxTemplate {
    fn cxx_methods(&self, schema: &Schema) -> Result<Vec<CxxMethod>, anyhow::Error> {
        let res = schema
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

        let mut mod_members = vec![format!(
            "static constexpr const char *kModuleName = \"{}\";",
            schema.module_name
        )];

        // Assign method metadata with function pointer to the TurboModule's method map
        //
        // ```cpp
        // methodMap_["multiply"] = MethodMetadata{1, &CxxMyTestModule::multiply};
        // ```
        let mut method_maps = cxx_methods
            .iter()
            .map(|method| format!("methodMap_[\"{}\"] = {};", method.name, method.metadata))
            .collect::<Vec<_>>();

        let mut method_defs = cxx_methods
            .iter()
            .map(|method| self.cxx_method_def(&method.name))
            .collect::<Vec<_>>();

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
        let mut method_impls = cxx_methods
            .into_iter()
            .map(|method| method.impl_func)
            .collect::<Vec<_>>();

        let (register_stmt, unregister_stmt) = if schema.signals.len() > 0 {
            let register_stmt = formatdoc! {
                r#"
                uintptr_t id = reinterpret_cast<uintptr_t>(this);
                auto& registry = craby::signals::SignalManager::getInstance();
                registry.registerDelegate(id,
                                          std::bind(&{cxx_mod}::emit,
                                          this,
                                          std::placeholders::_1));"#,
                cxx_mod = cxx_mod,
            };

            let unregister_stmt = formatdoc! {
                r#"
                uintptr_t id = reinterpret_cast<uintptr_t>(this);
                auto& registry = craby::signals::SignalManager::getInstance();
                registry.unregisterDelegate(id);"#,
            };

            schema.signals.iter().for_each(|signal| {
                method_maps.push(formatdoc! {
                    r#"methodMap_["{signal_name}"] = MethodMetadata{{1, &{cxx_mod}::{signal_name}}};"#,
                    signal_name = signal.name,
                    cxx_mod = cxx_mod,
                });

                method_defs.push(formatdoc! {
                    r#"
                    static facebook::jsi::Value
                    {signal_name}(facebook::jsi::Runtime &rt,
                        facebook::react::TurboModule &turboModule,
                        const facebook::jsi::Value args[], size_t count);"#,
                    signal_name = signal.name,
                });

                method_impls.push(formatdoc! {
                    r#"
                    jsi::Value {cxx_mod}::{signal_name}(jsi::Runtime &rt,
                                          react::TurboModule &turboModule,
                                          const jsi::Value args[],
                                          size_t count) {{
                      auto &thisModule = static_cast<{cxx_mod} &>(turboModule);
                      auto callInvoker = thisModule.callInvoker_;

                      try {{
                        if (1 != count) {{
                          throw jsi::JSError(rt, "Expected 1 argument");
                        }}

                        auto callback = args[0].asObject(rt).asFunction(rt);
                        uint64_t listenerId = nextListenerId_++;
                        
                        auto callbackRef = std::make_shared<jsi::Function>(std::move(callback));
                        auto name = "{signal_name}";
                        
                        if (listenersMap_.find(name) == listenersMap_.end()) {{
                          listenersMap_[name] = std::vector<std::shared_ptr<jsi::Function>>();
                        }}
                        listenersMap_[name].push_back(callbackRef);

                        return jsi::Function::createFromHostFunction(
                            rt,
                            jsi::PropNameID::forAscii(rt, "cleanup"),
                            0,
                            [listenerId, callbackRef, name](jsi::Runtime& rt, const jsi::Value&, const jsi::Value*, size_t) -> jsi::Value {{
                                std::lock_guard<std::mutex> lock(mutex_);
                                
                                auto& listeners = listenersMap_[name];
                                listeners.erase(
                                    std::remove_if(listeners.begin(), listeners.end(),
                                        [&callbackRef](const std::shared_ptr<jsi::Function>& fn) {{
                                            return fn.get() == callbackRef.get();
                                        }}),
                                    listeners.end()
                                );

                                if (listeners.empty()) {{
                                    listenersMap_.erase(name);
                                }}

                                return jsi::Value::undefined();
                            }}
                        );
                      }} catch (const jsi::JSError &err) {{
                        throw err;
                      }} catch (const std::exception &err) {{
                        throw jsi::JSError(rt, errorMessage(err));
                      }}
                    }}"#,
                    cxx_mod = cxx_mod,
                    signal_name = signal.name,
                });
            });

            mod_members.extend(vec![
              format!("inline static std::mutex mutex_;"),
              format!("inline static std::atomic<uint64_t> nextListenerId_;"),
              format!("inline static std::unordered_map<std::string, std::vector<std::shared_ptr<facebook::jsi::Function>>> listenersMap_;"),
            ]);

            method_defs.insert(0, "void emit(std::string name);".to_string());

            method_impls.insert(
                0,
                formatdoc! {
                r#"
                void {cxx_mod}::emit(std::string name) {{
                  std::vector<std::shared_ptr<facebook::jsi::Function>> listeners;

                  {{
                    std::lock_guard<std::mutex> lock(mutex_);
                    auto it = listenersMap_.find(name);
                    if (it != listenersMap_.end()) {{
                      for (const auto& fn : it->second) {{
                        listeners.push_back(fn);
                      }}
                    }}
                  }}

                  for (auto& listener : listeners) {{
                    try {{
                      callInvoker_->invokeAsync([listener](jsi::Runtime &rt) {{
                        listener->call(rt);
                      }});
                    }} catch (const std::exception& err) {{
                      // Noop
                    }}
                  }}
                }}
                "#,
                },
            );

            (register_stmt, unregister_stmt)
        } else {
            (String::from("// No signals"), String::from("// No signals"))
        };

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
            {register_stmt}
              callInvoker_ = std::move(jsInvoker);
            
            {method_maps}
            }}

            {cxx_mod}::~{cxx_mod}() {{
            {unregister_stmt}
            }}
            
            {method_impls}
            
            }} // namespace {flat_name}"#,
            flat_name = flat_name,
            cxx_mod = cxx_mod,
            register_stmt = indent_str(register_stmt, 2),
            unregister_stmt = indent_str(unregister_stmt, 2),
            method_maps = indent_str(method_maps.join("\n"), 2),
            method_impls = method_impls.join("\n\n"),
        };

        let hpp = formatdoc! {
            r#"
            namespace {flat_name} {{

            class JSI_EXPORT {cxx_mod} : public facebook::react::TurboModule {{
            public:
            {mod_members}

              {cxx_mod}(std::shared_ptr<facebook::react::CallInvoker> jsInvoker);
              ~{cxx_mod}();

            {method_defs}

            protected:
              std::shared_ptr<facebook::react::CallInvoker> callInvoker_;
            }};

            }} // namespace {flat_name}"#,
            flat_name = flat_name,
            cxx_mod = cxx_mod,
            mod_members = indent_str(mod_members.join("\n"), 2),
            method_defs = indent_str(method_defs.join("\n\n"), 2),
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

    fn cxx_signals(&self) -> Result<String, anyhow::Error> {
        Ok(formatdoc! {
            r#"
            #pragma once

            #include "rust/cxx.h"
            #include <functional>
            #include <memory>
            #include <unordered_map>

            namespace craby {{
            namespace signals {{

            using Delegate = std::function<void(const std::string& signalName)>;

            class SignalManager {{
            public:
              static SignalManager& getInstance() {{
                static SignalManager instance;
                return instance;
              }}

              void emit(uintptr_t id, rust::Str name) const {{
                std::lock_guard<std::mutex> lock(mutex_);
                auto it = delegates_.find(id);
                if (it != delegates_.end()) {{
                  it->second(std::string(name));
                }}
              }}

              void registerDelegate(uintptr_t id, Delegate delegate) const {{
                std::lock_guard<std::mutex> lock(mutex_);
                delegates_.insert_or_assign(id, delegate);
              }}

              void unregisterDelegate(uintptr_t id) const {{
                std::lock_guard<std::mutex> lock(mutex_);
                delegates_.erase(id);
              }}

            private:
              SignalManager() = default;
              mutable std::unordered_map<uintptr_t, Delegate> delegates_;
              mutable std::mutex mutex_;
            }};

            inline const SignalManager& getSignalManager() {{
              return SignalManager::getInstance();
            }}

            }} // namespace signals
            }} // namespace craby"#,
        })
    }
}

impl Template for CxxTemplate {
    type FileType = CxxFileType;

    fn render(
        &self,
        project: &CodegenContext,
        file_type: &Self::FileType,
    ) -> Result<Vec<(PathBuf, String)>, anyhow::Error> {
        let res = match file_type {
            CxxFileType::Mod => project
                .schemas
                .iter()
                .map(|schema| -> Result<Vec<(PathBuf, String)>, anyhow::Error> {
                    let (cpp, hpp) = self.cxx_mod(schema)?;
                    let cxx_mod = cxx_mod_cls_name(&schema.module_name);
                    let cxx_base_path = cxx_dir(&project.root);
                    let files = vec![
                        (cxx_base_path.join(format!("{}.cpp", cxx_mod)), cpp),
                        (cxx_base_path.join(format!("{}.hpp", cxx_mod)), hpp),
                    ];
                    Ok(files)
                })
                .collect::<Result<Vec<_>, _>>()
                .map(|v| v.into_iter().flatten().collect())?,
            CxxFileType::BridgingHpp => vec![(
                cxx_dir(&project.root).join("bridging-generated.hpp"),
                self.cxx_bridging(&project.schemas)?,
            )],
            CxxFileType::SignalsH => {
                let has_signals = project
                    .schemas
                    .iter()
                    .any(|schema| schema.signals.len() > 0);

                if has_signals {
                    vec![(
                        cxx_bridge_include_dir(&project.root).join("signals.h"),
                        self.cxx_signals()?,
                    )]
                } else {
                    vec![]
                }
            }
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
    fn generate(&self, project: &CodegenContext) -> Result<Vec<GenerateResult>, anyhow::Error> {
        let template = self.template_ref();
        let res = [
            template.render(project, &CxxFileType::Mod)?,
            template.render(project, &CxxFileType::BridgingHpp)?,
            template.render(project, &CxxFileType::SignalsH)?,
        ]
        .into_iter()
        .flatten()
        .map(|(path, content)| GenerateResult {
            path,
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
    fn test_cxx_generator() {
        let ctx = get_codegen_context();
        let generator = CxxGenerator::new();
        let results = generator.generate(&ctx).unwrap();
        let result = results
            .iter()
            .map(|res| format!("{}\n{}", res.path.display(), res.content))
            .collect::<Vec<_>>()
            .join("\n\n");

        assert_snapshot!(result);
    }
}
