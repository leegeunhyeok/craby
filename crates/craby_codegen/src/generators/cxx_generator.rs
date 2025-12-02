use std::fs;

use craby_common::{
    constants::{cxx_bridge_include_dir, cxx_dir},
    utils::string::{camel_case, flat_case, pascal_case, snake_case},
};
use indoc::formatdoc;

use crate::{
    constants::specs::RESERVED_ARG_NAME_MODULE,
    platform::cxx::CxxMethod,
    types::{CodegenContext, CxxModuleName, CxxNamespace, Schema},
    utils::indent_str,
};

use super::types::{Generator, GeneratorInvoker, Template, TemplateResult};

pub struct CxxTemplate;
pub struct CxxGenerator;

pub enum CxxFileType {
    /// cpp/hpp files
    Mod,
    /// bridging-generated.hpp
    BridgingHpp,
    /// CrabyUtils.hpp
    UtilsHpp,
    /// CrabySignals.h
    SignalsH,
}

impl CxxTemplate {
    /// Converts schema methods to C++ method definitions.
    ///
    /// # Generated Code
    ///
    /// ```
    /// ```
    fn cxx_methods(
        &self,
        project_name: &str,
        schema: &Schema,
    ) -> Result<Vec<CxxMethod>, anyhow::Error> {
        let cxx_ns = CxxNamespace::from(project_name);
        let mod_name = CxxModuleName::from(&schema.module_name);
        let res = schema
            .methods
            .iter()
            .map(|spec| spec.as_cxx_method(&cxx_ns, &mod_name))
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
    fn cxx_method_def(&self, name: &str) -> String {
        let method_name = camel_case(name);
        formatdoc! {
            r#"
            static facebook::jsi::Value
            {method_name}(facebook::jsi::Runtime &rt,
                facebook::react::TurboModule &turboModule,
                const facebook::jsi::Value args[], size_t count);"#,
        }
    }

    /// Returns the complete cxx TurboModule source/header files.
    ///
    /// # Generated Code (CPP)
    ///
    /// ```cpp
    /// #include "CxxMyTestModule.hpp"
    /// #include "cxx.h"
    /// #include "bridging-generated.hpp"
    /// #include <thread>
    /// #include <react/bridging/Bridging.h>
    ///
    /// using namespace facebook;
    ///
    /// namespace craby {
    /// namespace myproject {
    /// namespace modules {
    ///
    /// CxxMyTestModule::CxxMyTestModule(
    ///     std::shared_ptr<react::CallInvoker> jsInvoker)
    ///     : TurboModule(CxxMyTestModule::kModuleName, jsInvoker) {
    ///   callInvoker_ = std::move(jsInvoker);
    ///   threadPool_ = std::make_shared<craby::utils::ThreadPool>(10);
    ///   methodMap_["multiply"] = MethodMetadata{2, &CxxMyTestModule::multiply};
    /// }
    /// jsi::Value CxxMyTestModule::multiply(jsi::Runtime &rt,
    ///                                       react::TurboModule &turboModule,
    ///                                       const jsi::Value args[],
    ///                                       size_t count) {
    ///   // ...
    /// }
    ///
    /// } // namespace modules
    /// } // namespace myproject
    /// } // namespace craby
    /// ```
    ///
    /// # Generated Code (HPP)
    ///
    /// ```cpp
    /// #pragma once
    ///
    /// #include "CrabyUtils.hpp"
    /// #include "ffi.rs.h"
    /// #include <ReactCommon/TurboModule.h>
    /// #include <jsi/jsi.h>
    /// #include <memory>
    ///
    /// namespace craby {
    /// namespace myproject {
    /// namespace modules {
    ///
    /// class JSI_EXPORT CxxMyTestModule : public facebook::react::TurboModule {
    /// public:
    ///   static constexpr const char *kModuleName = "MyTestModule";
    ///   static std::string dataPath;
    ///
    ///   CxxMyTestModule(std::shared_ptr<facebook::react::CallInvoker> jsInvoker);
    ///   ~CxxMyTestModule();
    ///
    ///   static facebook::jsi::Value
    ///   multiply(facebook::jsi::Runtime &rt,
    ///            facebook::react::TurboModule &turboModule,
    ///            const facebook::jsi::Value args[], size_t count);
    ///
    /// protected:
    ///   std::shared_ptr<facebook::react::CallInvoker> callInvoker_;
    ///   std::shared_ptr<craby::mymodule::bridging::MyTestModule> module_;
    /// };
    ///
    /// } // namespace modules
    /// } // namespace myproject
    /// } // namespace craby
    /// ```
    fn cxx_mod(
        &self,
        schema: &Schema,
        project_name: &str,
    ) -> Result<(String, String), anyhow::Error> {
        let cxx_ns = CxxNamespace::from(project_name);
        let cxx_mod = CxxModuleName::from(&schema.module_name);
        let project_ns = flat_case(project_name);
        let cxx_methods = self.cxx_methods(project_name, schema)?;
        let include_stmt = format!("#include \"{cxx_mod}.hpp\"");

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

        let (register_stmt, unregister_stmt) = if !schema.signals.is_empty() {
            // Get signal enum type
            let signal_enum_name = if !schema.signals.is_empty() {
                Some(format!("{}Signal", schema.module_name))
            } else {
                None
            };
            
            let register_stmt = if let Some(ref signal_enum) = signal_enum_name {
                formatdoc! {
                    r#"
                    uintptr_t id = reinterpret_cast<uintptr_t>(this);
                    auto& manager = {cxx_ns}::signals::SignalManager::getInstance();
                    manager.registerDelegate(id,
                      [this](const std::string& name, void* signal) {{
                        this->emit(name, reinterpret_cast<bridging::{signal_enum}*>(signal));
                      }}
                    );"#,
                    signal_enum = signal_enum,
                }
            } else {
                String::new()
            };

            let unregister_stmt = formatdoc! {
                r#"
                // Unregister from signal manager
                uintptr_t id = reinterpret_cast<uintptr_t>(this);
                auto& manager = {cxx_ns}::signals::SignalManager::getInstance();
                manager.unregisterDelegate(id);"#,
            };

            for signal in &schema.signals {
                let signal_name = &signal.name;
                let cxx_signal_name = camel_case(&signal.name);

                method_maps.push(formatdoc! {
                    r#"methodMap_["{signal_name}"] = MethodMetadata{{1, &{cxx_mod}::{cxx_signal_name}}};"#,
                });

                method_defs.push(formatdoc! {
                    r#"
                    static facebook::jsi::Value
                    {signal_name}(facebook::jsi::Runtime &rt,
                        facebook::react::TurboModule &turboModule,
                        const facebook::jsi::Value args[], size_t count);"#,
                });

                method_impls.push(formatdoc! {
                    r#"
                    jsi::Value {cxx_mod}::{cxx_signal_name}(jsi::Runtime &rt,
                                          react::TurboModule &turboModule,
                                          const jsi::Value args[],
                                          size_t count) {{
                      auto &thisModule = static_cast<{cxx_mod} &>(turboModule);
                      auto callInvoker = thisModule.callInvoker_;
                      auto {it} = thisModule.module_;

                      try {{
                        if (1 != count) {{
                          throw jsi::JSError(rt, "Expected 1 argument");
                        }}

                        auto callback = args[0].asObject(rt).asFunction(rt);
                        auto callbackRef = std::make_shared<jsi::Function>(std::move(callback));
                        auto id = thisModule.nextListenerId_.fetch_add(1);
                        auto name = "{signal_name}";

                        if (thisModule.listenersMap_.find(name) == thisModule.listenersMap_.end()) {{
                          thisModule.listenersMap_[name] = std::unordered_map<size_t, std::shared_ptr<facebook::jsi::Function>>();
                        }}

                        {{
                          std::lock_guard<std::mutex> lock(thisModule.listenersMutex_);
                          thisModule.listenersMap_[name].emplace(id, callbackRef);
                        }}

                        auto modulePtr = &thisModule;
                        auto cleanup = [modulePtr, name, id] {{
                          std::lock_guard<std::mutex> lock(modulePtr->listenersMutex_);
                          auto eventMap = modulePtr->listenersMap_.find(name);
                          if (eventMap != modulePtr->listenersMap_.end()) {{
                            auto it = eventMap->second.find(id);
                            if (it != eventMap->second.end()) {{
                              eventMap->second.erase(it);
                            }}
                          }}
                          return jsi::Value::undefined();
                        }};

                        return jsi::Function::createFromHostFunction(
                          rt,
                          jsi::PropNameID::forAscii(rt, "cleanup"),
                          0,
                          [cleanup](jsi::Runtime& rt, const jsi::Value&, const jsi::Value*, size_t) -> jsi::Value {{
                            return cleanup();
                          }}
                        );
                      }} catch (const jsi::JSError &err) {{
                        throw err;
                      }} catch (const std::exception &err) {{
                        throw jsi::JSError(rt, {cxx_ns}::utils::errorMessage(err));
                      }}
                    }}"#,
                    it = RESERVED_ARG_NAME_MODULE,
                });
            }

            let signal_enum_name = if !schema.signals.is_empty() {
                Some(format!("{}Signal", schema.module_name))
            } else {
                None
            };
            
            method_defs.insert(0, if let Some(ref signal_enum) = signal_enum_name {
              format!("void emit(std::string name, bridging::{}* signal);", signal_enum)
            } else {
                "void emit(std::string name);".to_string()
            });

            // Generate payload extraction conditions dynamically
            let payload_extraction = if signal_enum_name.is_some() {
                let mut conditions: Vec<String> = schema.signals
                    .iter()
                    .filter_map(|signal| {
                        signal.payload_type.as_ref().map(|_| {
                            let function_name = format!("get_{}_payload", snake_case(&signal.name));
                            formatdoc! {
                                r#"else if (name == "{signal_name}") {{
                                  auto payload = craby::{project_ns}::bridging::{function_name}(*signalPtr);
                                  data = react::bridging::toJs(rt, payload);
                                }}"#,
                                signal_name = signal.name,
                                function_name = function_name,
                            }
                        })
                    })
                    .collect();
                
                if !conditions.is_empty() {
                    // Replace first "else if" with "if"
                    if let Some(first) = conditions.first_mut() {
                        *first = first.replace("else if", "if");
                    }
                    let joined = conditions.join(" ");
                    indent_str(&joined, 10)
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            method_impls.insert(
                0,
                if let Some(ref signal_enum) = signal_enum_name {
                    formatdoc! {
                        r#"
                        void {cxx_mod}::emit(std::string name, bridging::{signal_enum}* signal) {{
                          std::vector<std::shared_ptr<facebook::jsi::Function>> listeners;
                          {{
                            std::lock_guard<std::mutex> lock(listenersMutex_);
                            auto it = listenersMap_.find(name);
                            if (it != listenersMap_.end()) {{
                              for (auto &[_, listener] : it->second) {{
                                listeners.push_back(listener);
                              }}
                            }}
                          }}

                          // Prepare payload: extract from signal or use undefined
                          auto payloadPtr = std::make_shared<facebook::jsi::Value>();
                          
                          if (signal == nullptr) {{
                            *payloadPtr = facebook::jsi::Value::undefined();
                          }} else {{
                            // Use shared_ptr to manage signal lifetime across async callbacks
                            auto signalPtr = std::shared_ptr<bridging::{signal_enum}>(
                              signal,
                              [](bridging::{signal_enum}* ptr) {{
                                // Use Rust FFI function to drop signal memory
                                if (ptr != nullptr) {{
                                  craby::{project_ns}::bridging::drop_signal(ptr);
                                }}
                              }}
                            );

                            // Extract payload using FFI function and convert to jsi::Value
                            // We'll need to capture signalPtr in the lambda
                            for (auto& listener : listeners) {{
                              try {{
                                callInvoker_->invokeAsync([listener, signalPtr, name](jsi::Runtime &rt) {{
                                  jsi::Value data = jsi::Value::undefined();
                        {payload_extraction}
                                  listener->call(rt, data);
                                }});
                              }} catch (const std::exception& err) {{
                                // Noop
                              }}
                            }}
                            return;
                          }}

                          for (auto& listener : listeners) {{
                            try {{
                              callInvoker_->invokeAsync([listener, payloadPtr](jsi::Runtime &rt) {{
                                try {{
                                  listener->call(rt, *payloadPtr);
                                }} catch (const jsi::JSError &err) {{
                                  throw err;
                                }} catch (const std::exception &err) {{
                                  throw jsi::JSError(rt, {cxx_ns}::utils::errorMessage(err));
                                }}
                              }});
                            }} catch (const std::exception& err) {{
                              // Noop
                            }}
                          }}
                        }}"#,
                        signal_enum = signal_enum,
                        project_ns = project_ns,
                        cxx_mod = cxx_mod,
                        cxx_ns = cxx_ns,
                        payload_extraction = payload_extraction,
                    }
                } else {
                    formatdoc! {
                        r#"
                        void {cxx_mod}::emit(std::string name) {{
                          std::vector<std::shared_ptr<facebook::jsi::Function>> listeners;
                          {{
                            std::lock_guard<std::mutex> lock(listenersMutex_);
                            auto it = listenersMap_.find(name);
                            if (it != listenersMap_.end()) {{
                              for (auto &[_, listener] : it->second) {{
                                listeners.push_back(listener);
                              }}
                            }}
                          }}

                          for (auto& listener : listeners) {{
                            try {{
                              callInvoker_->invokeAsync([listener, payloadPtr](jsi::Runtime &rt) {{
                                try {{
                                  listener->call(rt, *payloadPtr);
                                }} catch (const jsi::JSError &err) {{
                                  throw err;
                                }} catch (const std::exception &err) {{
                                  throw jsi::JSError(rt, {cxx_ns}::utils::errorMessage(err));
                                }}
                              }});
                            }} catch (const std::exception& err) {{
                              // Noop
                            }}
                          }}
                        }}"#,
                    }
                }
            );


            (register_stmt, unregister_stmt)
        } else {
            (String::from("// No signals"), String::from("// No signals"))
        };

        let rs_module_name = pascal_case(&schema.module_name);
        let register_stmts = indent_str(&register_stmt, 2);
        let unregister_stmts = indent_str(&unregister_stmt, 2);
        let method_mapping_stmts = indent_str(&method_maps.join("\n"), 2);
        let method_impls = method_impls.join("\n\n");
        let cpp = formatdoc! {
            r#"
            std::string {cxx_mod}::dataPath = std::string();

            {cxx_mod}::{cxx_mod}(
                std::shared_ptr<react::CallInvoker> jsInvoker)
                : TurboModule({cxx_mod}::kModuleName, jsInvoker) {{
            {register_stmts}
              callInvoker_ = std::move(jsInvoker);
              module_ = std::shared_ptr<{cxx_ns}::bridging::{rs_module_name}>(
                {cxx_ns}::bridging::create{rs_module_name}(
                  reinterpret_cast<uintptr_t>(this),
                  rust::Str(dataPath.data(), dataPath.size())).into_raw(),
                []({cxx_ns}::bridging::{rs_module_name} *ptr) {{ rust::Box<{cxx_ns}::bridging::{rs_module_name}>::from_raw(ptr); }}
              );
              threadPool_ = std::make_shared<{cxx_ns}::utils::ThreadPool>(10);
            {method_mapping_stmts}
            }}

            {cxx_mod}::~{cxx_mod}() {{
              invalidate();
            }}

            void {cxx_mod}::invalidate() {{
              if (invalidated_.exchange(true)) {{
                return;
              }}

              invalidated_.store(true);
              listenersMap_.clear();
            
            {unregister_stmts}

              // Shutdown thread pool
              threadPool_->shutdown();
            }}
            
            {method_impls}"#,
        };

        let method_defs = indent_str(&method_defs.join("\n\n"), 2);
        let hpp = formatdoc! {
            r#"
            class JSI_EXPORT {cxx_mod} : public facebook::react::TurboModule {{
            public:
              static constexpr const char *kModuleName = "{turbo_module_name}";
              static std::string dataPath;

              {cxx_mod}(std::shared_ptr<facebook::react::CallInvoker> jsInvoker);
              ~{cxx_mod}();

              void invalidate();
            {method_defs}

            protected:
              std::shared_ptr<facebook::react::CallInvoker> callInvoker_;
              std::shared_ptr<{cxx_ns}::bridging::{rs_module_name}> module_;
              std::atomic<bool> invalidated_{{false}};
              std::atomic<size_t> nextListenerId_{{0}};
              std::mutex listenersMutex_;
              std::unordered_map<
                std::string,
                std::unordered_map<size_t, std::shared_ptr<facebook::jsi::Function>>>
                listenersMap_;
              std::shared_ptr<{cxx_ns}::utils::ThreadPool> threadPool_;
            }};"#,
            turbo_module_name = schema.module_name,
        };

        let cpp_content = formatdoc! {
            r#"
            {include_stmt}
            #include "cxx.h"
            #include "bridging-generated.hpp"
            #include <react/bridging/Bridging.h>

            using namespace facebook;

            namespace craby {{
            namespace {project_ns} {{
            namespace modules {{

            {cpp}

            }} // namespace modules
            }} // namespace {project_ns}
            }} // namespace craby"#,
        };

        let hpp_content = formatdoc! {
            r#"
            #pragma once

            #include "CrabyUtils.hpp"
            #include "ffi.rs.h"
            #include <ReactCommon/TurboModule.h>
            #include <jsi/jsi.h>
            #include <memory>
            
            namespace craby {{
            namespace {project_ns} {{
            namespace modules {{

            {hpp}

            }} // namespace modules
            }} // namespace {project_ns}
            }} // namespace craby"#,
        };

        Ok((cpp_content, hpp_content))
    }

    /// Generates C++ React Native bridging templates for custom types.
    ///
    /// # Generated Code
    ///
    /// ```cpp
    /// #pragma once
    ///
    /// #include "cxx.h"
    /// #include "ffi.rs.h"
    /// #include <react/bridging/Bridging.h>
    ///
    /// using namespace facebook;
    ///
    /// namespace facebook {
    /// namespace react {
    ///
    /// template <>
    /// struct Bridging<rust::String> {
    ///   static rust::String fromJs(jsi::Runtime& rt, const jsi::Value &value, std::shared_ptr<CallInvoker> callInvoker) {
    ///     auto str = value.asString(rt).utf8(rt);
    ///     return rust::String(str);
    ///   }
    ///
    ///   static jsi::Value toJs(jsi::Runtime& rt, const rust::String& value) {
    ///     return react::bridging::toJs(rt, std::string(value));
    ///   }
    /// };
    ///
    /// // Additional bridging templates for custom types...
    ///
    /// } // namespace react
    /// } // namespace facebook
    /// ```
    fn cxx_bridging(&self, ctx: &CodegenContext) -> Result<String, anyhow::Error> {
        let bridging_templates = ctx
            .schemas
            .iter()
            .flat_map(|schema| schema.as_cxx_bridging_templates(&ctx.project_name))
            .flatten()
            .collect::<Vec<_>>();

        let cxx_bridging = formatdoc! {
            r#"
            #pragma once

            #include "cxx.h"
            #include "ffi.rs.h"
            #include <react/bridging/Bridging.h>
            #include <variant>

            using namespace facebook;

            namespace {flat_name} {{

            class RustVecBuffer : public jsi::MutableBuffer {{
            public:
              explicit RustVecBuffer(rust::Vec<uint8_t> vec)
                : vec_(std::move(vec)) {{}}

              ~RustVecBuffer() override = default;

              size_t size() const override {{
                return vec_.size();
              }}

              uint8_t* data() override {{
                return const_cast<uint8_t*>(vec_.data());
              }}

            private:
              rust::Vec<uint8_t> vec_;
            }};

            }} // namespace {flat_name}

            namespace facebook {{
            namespace react {{

            template <>
            struct Bridging<std::monostate> {{
              static std::monostate fromJs(jsi::Runtime& rt, const jsi::Value &value, std::shared_ptr<CallInvoker> callInvoker) {{
                return std::monostate{{}};
              }}

              static jsi::Value toJs(jsi::Runtime& rt, const std::monostate& value) {{
                return jsi::Value::undefined();
              }}
            }};

            template <>
            struct Bridging<rust::Str> {{
              static rust::Str fromJs(jsi::Runtime& rt, const jsi::Value &value, std::shared_ptr<CallInvoker> callInvoker) {{
                auto str = value.asString(rt).utf8(rt);
                return rust::Str(str.data(), str.size());
              }}

              static jsi::Value toJs(jsi::Runtime& rt, const rust::Str& value) {{
                return react::bridging::toJs(rt, std::string(value.data(), value.size()));
              }}
            }};

            template <>
            struct Bridging<rust::String> {{
              static rust::String fromJs(jsi::Runtime& rt, const jsi::Value &value, std::shared_ptr<CallInvoker> callInvoker) {{
                auto str = value.asString(rt).utf8(rt);
                return rust::String(str.data(), str.size());
              }}

              static jsi::Value toJs(jsi::Runtime& rt, const rust::String& value) {{
                return react::bridging::toJs(rt, std::string(value.data(), value.size()));
              }}
            }};

            template <>
            struct Bridging<rust::Vec<uint8_t>> {{
              static rust::Vec<uint8_t> fromJs(jsi::Runtime& rt, const jsi::Value &value, std::shared_ptr<CallInvoker> callInvoker) {{
                auto arrayBuffer = value.asObject(rt).getArrayBuffer(rt);
                uint8_t* data = arrayBuffer.data(rt);
                size_t size = arrayBuffer.size(rt);
                rust::Vec<uint8_t> vec;
                vec.reserve(size);

                std::memcpy(vec.data(), data, size);

                return vec;
              }}

              static jsi::Value toJs(jsi::Runtime& rt, const rust::Vec<uint8_t>& vec) {{
                auto buffer = std::make_shared<{flat_name}::RustVecBuffer>(std::move(vec));
                return jsi::ArrayBuffer(rt, buffer);
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
            flat_name = flat_case(&ctx.project_name),
            bridging_templates = if bridging_templates.is_empty() { "".to_string() } else { format!("\n{}\n", bridging_templates.join("\n\n")) },
        };

        Ok(cxx_bridging)
    }

    /// Generates C++ utils header file.
    ///
    /// # Generated Code
    ///
    /// ```cpp
    /// #pragma once
    ///
    /// #include "cxx.h"
    /// #include "ffi.rs.h"
    /// #include <condition_variable>
    /// #include <functional>
    /// #include <mutex>
    /// #include <queue>
    /// #include <thread>
    /// #include <vector>
    ///
    /// namespace craby {
    /// namespace mymodule {
    /// namespace utils {
    ///
    /// class ThreadPool {
    /// private:
    ///   bool stop;
    ///   std::mutex mutex;
    ///   std::condition_variable condition;
    ///   std::queue<std::function<void()>> tasks;
    ///   std::vector<std::thread> workers;
    /// }
    ///
    /// public:
    ///   ThreadPool(size_t num_threads = 10) : stop(false) {
    ///     for (size_t i = 0; i < num_threads; ++i) {
    ///       workers.emplace_back([this] {
    ///         while (true) {
    ///           std::function<void()> task;
    ///
    ///           {
    ///             std::unique_lock<std::mutex> lock(this->mutex);
    ///             this->condition.wait(
    ///                 lock, [this] { return this->stop || !this->tasks.empty(); });
    ///
    ///           if (this->stop && this->tasks.empty()) {
    ///             return;
    ///           }
    ///
    ///           task = std::move(this->tasks.front());
    ///           this->tasks.pop();
    ///         }
    ///
    ///         task();
    ///       }
    ///     });
    ///   }
    ///
    ///   template <class F> void enqueue(F &&f) {
    ///     {
    ///       std::unique_lock<std::mutex> lock(mutex);
    ///       if (stop) {
    ///         return;
    ///       }
    ///       tasks.emplace(std::forward<F>(f));
    ///     }
    ///     condition.notify_one();
    ///   }
    ///
    ///   void shutdown() {
    ///     {
    ///       std::unique_lock<std::mutex> lock(mutex);
    ///       stop = true;
    ///       std::queue<std::function<void()>> empty;
    ///       std::swap(tasks, empty);
    ///     }
    ///
    ///     condition.notify_all();
    ///
    ///     for (std::thread &worker : workers) {
    ///       if (worker.joinable()) {
    ///         worker.join();
    ///       }
    ///     }
    ///   }
    ///
    ///   ~ThreadPool() {
    ///     shutdown();
    ///   }
    /// };
    ///
    /// inline std::string errorMessage(const std::exception &err) {
    ///   const auto* rs_err = dynamic_cast<const rust::Error*>(&err);
    ///   return std::string(rs_err ? rs_err->what() : err.what());
    /// }
    ///
    /// } // namespace utils
    /// } // namespace mymodule
    /// } // namespace craby
    /// ```
    fn cxx_utils(&self, project_name: &str) -> Result<String, anyhow::Error> {
        let flat_name = flat_case(project_name);

        Ok(formatdoc! {
            r#"
            #pragma once

            #include "cxx.h"
            #include "ffi.rs.h"
            #include <condition_variable>
            #include <functional>
            #include <mutex>
            #include <queue>
            #include <thread>
            #include <vector>

            namespace craby {{
            namespace {flat_name} {{
            namespace utils {{

            class ThreadPool {{
            private:
              bool stop;
              std::mutex mutex;
              std::condition_variable condition;
              std::queue<std::function<void()>> tasks;
              std::vector<std::thread> workers;

            public:
              ThreadPool(size_t num_threads = 10) : stop(false) {{
                for (size_t i = 0; i < num_threads; ++i) {{
                  workers.emplace_back([this] {{
                    while (true) {{
                      std::function<void()> task;

                      {{
                        std::unique_lock<std::mutex> lock(this->mutex);
                        this->condition.wait(
                            lock, [this] {{ return this->stop || !this->tasks.empty(); }});

                        if (this->stop && this->tasks.empty()) {{
                          return;
                        }}

                        task = std::move(this->tasks.front());
                        this->tasks.pop();
                      }}

                      task();
                    }}
                  }});
                }}
              }}

              template <class F> void enqueue(F &&f) {{
                {{
                  std::unique_lock<std::mutex> lock(mutex);
                  if (stop) {{
                    return;
                  }}
                  tasks.emplace(std::forward<F>(f));
                }}
                condition.notify_one();
              }}

              void shutdown() {{
                {{
                  std::unique_lock<std::mutex> lock(mutex);
                  stop = true;
                  std::queue<std::function<void()>> empty;
                  std::swap(tasks, empty);
                }}

                condition.notify_all();

                for (std::thread &worker : workers) {{
                  if (worker.joinable()) {{
                    worker.join();
                  }}
                }}
              }}

              ~ThreadPool() {{
                shutdown();
              }}
            }};

            inline std::string errorMessage(const std::exception &err) {{
              const auto* rs_err = dynamic_cast<const rust::Error*>(&err);
              return std::string(rs_err ? rs_err->what() : err.what());
            }}

            }} // namespace utils
            }} // namespace {flat_name}
            }} // namespace craby"#,
        })
    }

    /// Generates the signal manager header file for event emission.
    ///
    /// # Generated Code
    ///
    /// ```cpp
    /// #pragma once
    ///
    /// #include "rust/cxx.h"
    /// #include <functional>
    /// #include <memory>
    /// #include <mutex>
    /// #include <unordered_map>
    ///
    /// namespace craby {
    /// namespace mymodule {
    /// namespace signals {
    ///
    /// class SignalManager {
    /// public:
    ///   static SignalManager& getInstance() {
    ///     static SignalManager instance;
    ///     return instance;
    ///   }
    ///
    ///   void emit(uintptr_t id, rust::Str name) const {
    ///     std::lock_guard<std::mutex> lock(mutex_);
    ///     auto it = delegates_.find(id);
    ///     if (it != delegates_.end()) {
    ///       it->second(std::string(name));
    ///     }
    ///   }
    ///
    ///   void registerDelegate(uintptr_t id, Delegate delegate) const {
    ///     std::lock_guard<std::mutex> lock(mutex_);
    ///     delegates_.insert_or_assign(id, delegate);
    ///   }
    ///
    ///   void unregisterDelegate(uintptr_t id) const {
    ///     std::lock_guard<std::mutex> lock(mutex_);
    ///     delegates_.erase(id);
    ///   }
    ///
    /// private:
    ///   SignalManager() = default;
    ///   mutable std::unordered_map<uintptr_t, Delegate> delegates_;
    ///   mutable std::mutex mutex_;
    /// };
    ///
    /// } // namespace signals
    /// } // namespace mymodule
    /// } // namespace craby
    /// ```
    fn cxx_signals(&self, project_name: &str, schemas: &[Schema]) -> Result<String, anyhow::Error> {
      let flat_name = flat_case(project_name);
      
      // Find schema with first signal
      let signal_schema = schemas.iter().find(|s| !s.signals.is_empty());
      let signal_enum = signal_schema.map(|s| format!("{}Signal", s.module_name));
      let cxx_mod = signal_schema.map(|s| format!("Cxx{}", pascal_case(&s.module_name)));
      
      Ok(formatdoc! {
          r#"
          #pragma once

          #include "rust/cxx.h"
          #include <functional>
          #include <memory>
          #include <mutex>
          #include <unordered_map>

          {forward_declarations}

          namespace craby {{
          namespace {flat_name} {{
          namespace signals {{

          {signal_delegate_typedef}

          class SignalManager {{
          public:
            static SignalManager& getInstance() {{
              static SignalManager instance;
              return instance;
            }}

            {emit_impl}

            {register_delegate_impl}

            void unregisterDelegate(uintptr_t id) const {{
              std::lock_guard<std::mutex> lock(mutex_);
              delegates_.erase(id);
            }}

          private:
            SignalManager() = default;
            {delegates_map}
            mutable std::mutex mutex_;
          }};

          inline const SignalManager& getSignalManager() {{
            return SignalManager::getInstance();
          }}

          }} // namespace signals
          }} // namespace {flat_name}
          }} // namespace craby"#,
          flat_name = flat_name,
          forward_declarations = if let (Some(ref enum_name), Some(ref mod_name)) = (&signal_enum, &cxx_mod) {
              formatdoc! {
                  r#"
                  namespace craby {{
                  namespace {flat_name} {{
                  namespace bridging {{
                    struct {enum_name};
                  }}
                  namespace modules {{
                    class {mod_name};
                  }}
                  }}
                  }}"#,
                  enum_name = enum_name,
                  mod_name = mod_name,
                  flat_name = flat_name
              }
          } else {
              String::new()
          },
          signal_delegate_typedef = if signal_enum.is_some() {
              formatdoc! {
                  r#"
                  using Delegate = std::function<void(const std::string& signalName, void* signal)>;"#
              }
          } else {
              String::new()
          },
          emit_impl = if let Some(ref enum_name) = signal_enum {
              formatdoc! {
                  r#"
                  void emit(uintptr_t id, rust::Str name, craby::{flat_name}::bridging::{enum_name}* signal) const {{
                      std::lock_guard<std::mutex> lock(mutex_);
                      auto it = delegates_.find(id);
                      if (it != delegates_.end()) {{
                        it->second(std::string(name), reinterpret_cast<void*>(signal));
                      }}
                    }}"#,
                  enum_name = enum_name,
                  flat_name = flat_name
              }
          } else {
              String::new()
          },
          register_delegate_impl = if signal_enum.is_some() {
              formatdoc! {
                  r#"
                  void registerDelegate(uintptr_t id, Delegate delegate) const {{
                      std::lock_guard<std::mutex> lock(mutex_);
                      delegates_.insert_or_assign(id, delegate);
                    }}"#
              }
          } else {
              String::new()
          },
          delegates_map = if signal_enum.is_some() {
              formatdoc! {
                  r#"
                  mutable std::unordered_map<uintptr_t, Delegate> delegates_;"#
              }
          } else {
              String::new()
          },
      })
  }
}

impl Template for CxxTemplate {
    type FileType = CxxFileType;

    fn render(
        &self,
        ctx: &CodegenContext,
        file_type: &Self::FileType,
    ) -> Result<Vec<TemplateResult>, anyhow::Error> {
        let res = match file_type {
            CxxFileType::Mod => ctx
                .schemas
                .iter()
                .map(|schema| -> Result<Vec<TemplateResult>, anyhow::Error> {
                    let (cpp, hpp) = self.cxx_mod(schema, &ctx.project_name)?;
                    let cxx_mod = CxxModuleName::from(&schema.module_name);
                    let cxx_base_path = cxx_dir(&ctx.root);
                    let files = vec![
                        TemplateResult {
                            path: cxx_base_path.join(format!("{cxx_mod}.cpp")),
                            content: cpp,
                            overwrite: true,
                        },
                        TemplateResult {
                            path: cxx_base_path.join(format!("{cxx_mod}.hpp")),
                            content: hpp,
                            overwrite: true,
                        },
                    ];
                    Ok(files)
                })
                .collect::<Result<Vec<_>, _>>()
                .map(|v| v.into_iter().flatten().collect())?,
            CxxFileType::BridgingHpp => vec![TemplateResult {
                path: cxx_dir(&ctx.root).join("bridging-generated.hpp"),
                content: self.cxx_bridging(ctx)?,
                overwrite: true,
            }],
            CxxFileType::UtilsHpp => vec![TemplateResult {
                path: cxx_dir(&ctx.root).join("CrabyUtils.hpp"),
                content: self.cxx_utils(&ctx.project_name)?,
                overwrite: true,
            }],
            CxxFileType::SignalsH => {
                let has_signals = ctx.schemas.iter().any(|schema| !schema.signals.is_empty());

                if has_signals {
                    vec![TemplateResult {
                        path: cxx_bridge_include_dir(&ctx.root).join("CrabySignals.h"),
                        content: self.cxx_signals(&ctx.project_name, &ctx.schemas)?,
                        overwrite: true,
                    }]
                } else {
                    Vec::default()
                }
            }
        };

        Ok(res)
    }
}

impl Default for CxxGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CxxGenerator {
    pub fn new() -> Self {
        Self {}
    }
}

impl Generator<CxxTemplate> for CxxGenerator {
    fn cleanup(ctx: &CodegenContext) -> Result<(), anyhow::Error> {
        let cxx_dir = cxx_dir(&ctx.root);

        if cxx_dir.try_exists()? {
            fs::read_dir(cxx_dir)?.try_for_each(|entry| -> Result<(), anyhow::Error> {
                let path = entry?.path();
                let file_name = path.file_name().unwrap().to_string_lossy().to_string();

                if file_name.starts_with("Cxx")
                    && (file_name.ends_with("Module.cpp") || file_name.ends_with("Module.hpp"))
                {
                    fs::remove_file(&path)?;
                }

                Ok(())
            })?;
        }

        Ok(())
    }

    fn generate(&self, ctx: &CodegenContext) -> Result<Vec<TemplateResult>, anyhow::Error> {
        let template = self.template_ref();
        let res = [
            template.render(ctx, &CxxFileType::Mod)?,
            template.render(ctx, &CxxFileType::BridgingHpp)?,
            template.render(ctx, &CxxFileType::UtilsHpp)?,
            template.render(ctx, &CxxFileType::SignalsH)?,
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        Ok(res)
    }

    fn template_ref(&self) -> &CxxTemplate {
        &CxxTemplate
    }
}

impl GeneratorInvoker for CxxGenerator {
    fn invoke_generate(&self, ctx: &CodegenContext) -> Result<Vec<TemplateResult>, anyhow::Error> {
        self.generate(ctx)
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
