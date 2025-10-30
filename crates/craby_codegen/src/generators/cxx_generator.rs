use std::{fs, path::PathBuf};

use craby_common::{
    constants::{cxx_bridge_include_dir, cxx_dir},
    utils::string::{camel_case, flat_case, pascal_case},
};
use indoc::formatdoc;

use crate::{
    constants::{cxx_mod_cls_name, specs::RESERVED_ARG_NAME_MODULE},
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
    fn cxx_method_def(&self, name: &str) -> String {
        formatdoc! {
            r#"
            static facebook::jsi::Value
            {name}(facebook::jsi::Runtime &rt,
                facebook::react::TurboModule &turboModule,
                const facebook::jsi::Value args[], size_t count);"#,
            name = camel_case(name),
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
    /// namespace mymodule {
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
    /// } // namespace mymodule
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
    /// namespace mymodule {
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
    ///   std::shared_ptr<craby::bridging::MyTestModule> module_;
    /// };
    ///
    /// } // namespace mymodule
    /// } // namespace craby
    /// ```
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
            let register_stmt = formatdoc! {
                r#"
                uintptr_t id = reinterpret_cast<uintptr_t>(this);
                auto& manager = craby::signals::SignalManager::getInstance();
                manager.registerDelegate(id,
                                         std::bind(&{cxx_mod}::emit,
                                         this,
                                         std::placeholders::_1));"#,
                cxx_mod = cxx_mod,
            };

            let unregister_stmt = formatdoc! {
                r#"
                // Unregister from signal manager
                uintptr_t id = reinterpret_cast<uintptr_t>(this);
                auto& manager = craby::signals::SignalManager::getInstance();
                manager.unregisterDelegate(id);"#,
            };

            for signal in &schema.signals {
                let cxx_signal_name = camel_case(&signal.name);

                method_maps.push(formatdoc! {
                    r#"methodMap_["{signal_name}"] = MethodMetadata{{1, &{cxx_mod}::{cxx_signal_name}}};"#,
                    signal_name = signal.name,
                    cxx_signal_name = cxx_signal_name,
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
                        throw jsi::JSError(rt, craby::utils::errorMessage(err));
                      }}
                    }}"#,
                    cxx_mod = cxx_mod,
                    it = RESERVED_ARG_NAME_MODULE,
                    signal_name = signal.name,
                    cxx_signal_name = cxx_signal_name,
                });
            }

            method_defs.insert(0, "void emit(std::string name);".to_string());

            method_impls.insert(
                0,
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
                          callInvoker_->invokeAsync([listener](jsi::Runtime &rt) {{
                            listener->call(rt);
                          }});
                        }} catch (const std::exception& err) {{
                          // Noop
                        }}
                      }}
                    }}"#,
                },
            );

            (register_stmt, unregister_stmt)
        } else {
            (String::from("// No signals"), String::from("// No signals"))
        };

        // ```cpp
        // namespace mymodule {
        //
        // std::string CxxMyTestModule::dataPath = std::string();
        //
        // CxxMyTestModule::CxxMyTestModule(
        //     std::shared_ptr<react::CallInvoker> jsInvoker)
        //     : TurboModule(CxxMyTestModule::kModuleName, jsInvoker) {
        //   callInvoker_ = std::move(jsInvoker);
        //   module_ = std::shared_ptr(...);
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

            std::string {cxx_mod}::dataPath = std::string();

            {cxx_mod}::{cxx_mod}(
                std::shared_ptr<react::CallInvoker> jsInvoker)
                : TurboModule({cxx_mod}::kModuleName, jsInvoker) {{
            {register_stmt}
              callInvoker_ = std::move(jsInvoker);
              module_ = std::shared_ptr<craby::bridging::{module_name}>(
                craby::bridging::create{module_name}(
                  reinterpret_cast<uintptr_t>(this),
                  rust::Str(dataPath.data(), dataPath.size())).into_raw(),
                [](craby::bridging::{module_name} *ptr) {{ rust::Box<craby::bridging::{module_name}>::from_raw(ptr); }}
              );
              threadPool_ = std::make_shared<craby::utils::ThreadPool>(10);
            {method_maps}
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
            
            {unregister_stmt}

              // Shutdown thread pool
              threadPool_->shutdown();
            }}
            
            {method_impls}
            
            }} // namespace {flat_name}"#,
            module_name = pascal_case(&schema.module_name),
            flat_name = flat_name,
            cxx_mod = cxx_mod,
            register_stmt = indent_str(&register_stmt, 2),
            unregister_stmt = indent_str(&unregister_stmt, 2),
            method_maps = indent_str(&method_maps.join("\n"), 2),
            method_impls = method_impls.join("\n\n"),
        };

        let hpp = formatdoc! {
            r#"
            namespace {flat_name} {{

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
              std::shared_ptr<craby::bridging::{module_name}> module_;
              std::atomic<bool> invalidated_{{false}};
              std::atomic<size_t> nextListenerId_{{0}};
              std::mutex listenersMutex_;
              std::unordered_map<
                std::string,
                std::unordered_map<size_t, std::shared_ptr<facebook::jsi::Function>>>
                listenersMap_;
              std::shared_ptr<craby::utils::ThreadPool> threadPool_;
            }};

            }} // namespace {flat_name}"#,
            turbo_module_name = schema.module_name,
            module_name = pascal_case(&schema.module_name),
            flat_name = flat_name,
            cxx_mod = cxx_mod,
            method_defs = indent_str(&method_defs.join("\n\n"), 2),
        };

        // ```cpp
        // #include "my_module.hpp"
        // #include "cxx.h"
        // #include "bridging-generated.hpp"
        // #include <thread>
        // #include <react/bridging/Bridging.h>
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
            #include "cxx.h"
            #include "bridging-generated.hpp"
            #include <react/bridging/Bridging.h>

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

            #include "CrabyUtils.hpp"
            #include "ffi.rs.h"
            #include <ReactCommon/TurboModule.h>
            #include <jsi/jsi.h>
            #include <memory>
            
            namespace craby {{
            {hpp}
            }} // namespace craby"#,
            hpp = hpp,
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
    fn cxx_bridging(&self, schemas: &[Schema]) -> Result<String, anyhow::Error> {
        let bridging_templates = schemas
            .iter()
            .flat_map(|schema| schema.as_cxx_bridging_templates())
            .flatten()
            .collect::<Vec<_>>();

        let cxx_bridging = formatdoc! {
            r#"
            #pragma once

            #include "cxx.h"
            #include "ffi.rs.h"
            #include <react/bridging/Bridging.h>

            using namespace facebook;

            namespace facebook {{
            namespace react {{

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
    /// } // namespace craby
    /// ```
    fn cxx_utils(&self) -> String {
        formatdoc! {
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
            }} // namespace craby"#
        }
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
    /// namespace signals {
    ///
    /// using Delegate = std::function<void(const std::string& signalName)>;
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
    /// } // namespace craby
    /// ```
    fn cxx_signals(&self) -> Result<String, anyhow::Error> {
        Ok(formatdoc! {
            r#"
            #pragma once

            #include "rust/cxx.h"
            #include <functional>
            #include <memory>
            #include <mutex>
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
        ctx: &CodegenContext,
        file_type: &Self::FileType,
    ) -> Result<Vec<(PathBuf, String)>, anyhow::Error> {
        let res = match file_type {
            CxxFileType::Mod => ctx
                .schemas
                .iter()
                .map(|schema| -> Result<Vec<(PathBuf, String)>, anyhow::Error> {
                    let (cpp, hpp) = self.cxx_mod(schema)?;
                    let cxx_mod = cxx_mod_cls_name(&schema.module_name);
                    let cxx_base_path = cxx_dir(&ctx.root);
                    let files = vec![
                        (cxx_base_path.join(format!("{}.cpp", cxx_mod)), cpp),
                        (cxx_base_path.join(format!("{}.hpp", cxx_mod)), hpp),
                    ];
                    Ok(files)
                })
                .collect::<Result<Vec<_>, _>>()
                .map(|v| v.into_iter().flatten().collect())?,
            CxxFileType::BridgingHpp => vec![(
                cxx_dir(&ctx.root).join("bridging-generated.hpp"),
                self.cxx_bridging(&ctx.schemas)?,
            )],
            CxxFileType::UtilsHpp => {
                vec![(cxx_dir(&ctx.root).join("CrabyUtils.hpp"), self.cxx_utils())]
            }
            CxxFileType::SignalsH => {
                let has_signals = ctx.schemas.iter().any(|schema| !schema.signals.is_empty());

                if has_signals {
                    vec![(
                        cxx_bridge_include_dir(&ctx.root).join("CrabySignals.h"),
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

    fn generate(&self, ctx: &CodegenContext) -> Result<Vec<GenerateResult>, anyhow::Error> {
        let template = self.template_ref();
        let res = [
            template.render(ctx, &CxxFileType::Mod)?,
            template.render(ctx, &CxxFileType::BridgingHpp)?,
            template.render(ctx, &CxxFileType::UtilsHpp)?,
            template.render(ctx, &CxxFileType::SignalsH)?,
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
    fn invoke_generate(&self, ctx: &CodegenContext) -> Result<Vec<GenerateResult>, anyhow::Error> {
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
