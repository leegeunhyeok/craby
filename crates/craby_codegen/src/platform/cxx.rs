use craby_common::utils::string::flat_case;
use indoc::formatdoc;
use template::{cxx_arg_ref, cxx_arg_var};

use crate::{
    constants::cxx_mod_cls_name,
    types::schema::{FunctionSpec, TypeAnnotation},
    utils::indent_str,
};

#[derive(Debug, Clone)]
pub struct CxxMethod {
    pub name: String,
    pub metadata: String,
    pub impl_func: String,
}

pub trait ToCxxType {
    fn to_cxx_type(&self, mod_name: &String) -> Result<String, anyhow::Error>;
}

pub trait ToCxxBridging {
    fn to_cxx(&self, mod_name: &String, ident: &String) -> Result<String, anyhow::Error>;
    fn to_js(&self) -> Result<String, anyhow::Error>;
}

pub trait ToCxxMethod {
    /// Returns the cxx function's metadata and implementation for the `FunctionSpec`.
    ///
    /// ```cpp
    /// // metadata
    /// MethodMetadata{1, &CxxMyTestModule::myFunc}
    ///
    /// // impl function
    /// jsi::Value CxxMyTestModule::myFunc(jsi::Runtime &rt,
    ///                                    react::TurboModule &turboModule,
    ///                                    const jsi::Value args[],
    ///                                    size_t count) {
    ///     // Implementation
    /// }
    /// ```
    fn to_cxx_method(&self, mod_name: &String) -> Result<CxxMethod, anyhow::Error>;
}

impl ToCxxType for TypeAnnotation {
    fn to_cxx_type(&self, mod_name: &String) -> Result<String, anyhow::Error> {
        let cxx_type = match self {
            // Boolean type
            TypeAnnotation::BooleanTypeAnnotation => "bool".to_string(),

            // Number types
            TypeAnnotation::NumberTypeAnnotation
            | TypeAnnotation::FloatTypeAnnotation
            | TypeAnnotation::DoubleTypeAnnotation
            | TypeAnnotation::Int32TypeAnnotation
            | TypeAnnotation::NumberLiteralTypeAnnotation { .. } => "double".to_string(),

            // String types
            TypeAnnotation::StringTypeAnnotation
            | TypeAnnotation::StringLiteralTypeAnnotation { .. }
            | TypeAnnotation::StringLiteralUnionTypeAnnotation { .. } => "std::string".to_string(),

            // Array type
            TypeAnnotation::ArrayTypeAnnotation { element_type } => {
                format!("rust::Vec<{}>", element_type.to_cxx_type(mod_name)?)
            }

            // Enum
            TypeAnnotation::EnumDeclaration { name, .. } => {
                format!("craby::{}::{}", flat_case(mod_name), name)
            }

            // Type alias
            TypeAnnotation::TypeAliasTypeAnnotation { name } => {
                format!("craby::{}::{}", flat_case(mod_name), name)
            }

            // Unsupported types with message
            TypeAnnotation::FunctionTypeAnnotation { .. } => {
                return Err(anyhow::anyhow!(
                    "Function type annotation is not supported: {:?}",
                    self
                ));
            }
            TypeAnnotation::ObjectTypeAnnotation { .. } => {
                return Err(anyhow::anyhow!(
                    "Use strict type alias instead of object type: {:?}",
                    self
                ));
            }

            // Unsupported types
            _ => {
                return Err(anyhow::anyhow!("Unsupported type annotation: {:?}", self));
            }
        };

        Ok(cxx_type)
    }
}

impl ToCxxBridging for TypeAnnotation {
    fn to_cxx(&self, mod_name: &String, ident: &String) -> Result<String, anyhow::Error> {
        let is_supported = match &*self {
            // Boolean type
            TypeAnnotation::BooleanTypeAnnotation
            // Number types
            | TypeAnnotation::NumberTypeAnnotation { .. }
            | TypeAnnotation::FloatTypeAnnotation { .. }
            | TypeAnnotation::DoubleTypeAnnotation { .. }
            | TypeAnnotation::Int32TypeAnnotation { .. }
            | TypeAnnotation::NumberLiteralTypeAnnotation { .. }
            // String types
            | TypeAnnotation::StringTypeAnnotation { .. }
            | TypeAnnotation::StringLiteralTypeAnnotation { .. }
            | TypeAnnotation::StringLiteralUnionTypeAnnotation { .. }
            // Array type
            | TypeAnnotation::ArrayTypeAnnotation { .. }
            // Enum type
            | TypeAnnotation::EnumDeclaration { .. }
            // Type alias (Object)
            | TypeAnnotation::TypeAliasTypeAnnotation { .. } => true,
            _ => false,
        };

        if is_supported {
            let cxx_type = self.to_cxx_type(mod_name)?;
            Ok(format!(
                "react::bridging::fromJs<{}>(rt, {}, callInvoker)",
                cxx_type, ident
            ))
        } else {
            Err(anyhow::anyhow!("Unsupported type annotation: {:?}", self))
        }
    }

    fn to_js(&self) -> Result<String, anyhow::Error> {
        let to_js = match &*self {
            // Boolean type
            TypeAnnotation::BooleanTypeAnnotation
            // Number types
            | TypeAnnotation::NumberTypeAnnotation { .. }
            | TypeAnnotation::FloatTypeAnnotation { .. }
            | TypeAnnotation::DoubleTypeAnnotation { .. }
            | TypeAnnotation::Int32TypeAnnotation { .. }
            | TypeAnnotation::NumberLiteralTypeAnnotation { .. }
            // String types
            | TypeAnnotation::StringTypeAnnotation { .. }
            | TypeAnnotation::StringLiteralTypeAnnotation { .. }
            | TypeAnnotation::StringLiteralUnionTypeAnnotation { .. }
            // Array type
            | TypeAnnotation::ArrayTypeAnnotation { .. }
            // Enum type
            | TypeAnnotation::EnumDeclaration { .. }
            // Type alias (Object)
            | TypeAnnotation::TypeAliasTypeAnnotation { .. } => format!("react::bridging::toJs(rt, ret)"),
            TypeAnnotation::PromiseTypeAnnotation { .. } => format!("react::bridging::toJs(rt, promise)"),
            _ => {
              return Err(anyhow::anyhow!("Unsupported type annotation: {:?}", self));
            },
        };

        Ok(to_js)
    }
}

impl ToCxxMethod for FunctionSpec {
    fn to_cxx_method(&self, mod_name: &String) -> Result<CxxMethod, anyhow::Error> {
        let (args_decls, invoke_stmts) = if let TypeAnnotation::FunctionTypeAnnotation {
            return_type_annotation,
            params,
        } = &*self.type_annotation
        {
            let mut args = vec![];
            let mut args_decls = vec![];

            for (idx, param) in params.iter().enumerate() {
                let arg_ref = cxx_arg_ref(idx);
                let arg_var = cxx_arg_var(idx);
                let cxx_type = param.type_annotation.to_cxx(mod_name, &arg_ref)?;
                args.push(arg_var.clone());
                args_decls.push(format!("auto {} = {};", arg_var, cxx_type));
            }

            let invoke_stmts = match &**return_type_annotation {
                TypeAnnotation::PromiseTypeAnnotation { element_type } => {
                    let fn_args = args.join(", ");
                    let mut bind_args = vec!["promise".to_string()];
                    bind_args.extend(args);

                    formatdoc! {
                        r#"
                        react::AsyncPromise<{ret_type}> promise(rt, callInvoker);
      
                        std::thread([{bind_args}]() mutable {{
                          try {{
                            auto ret = craby::{flat_name}::{fn_name}({fn_args});
                            promise.resolve(ret);
                          }} catch (const jsi::JSError &err) {{
                            promise.reject(err.getMessage());
                          }} catch (const std::exception &err) {{
                            promise.reject(craby::helpers::errorMessage(err));
                          }}
                        }}).detach();
                        
                        return {ret};"#,
                        bind_args = bind_args.join(", "),
                        fn_name = self.name,
                        fn_args = fn_args,
                        flat_name = flat_case(mod_name),
                        ret_type = element_type.to_cxx_type(mod_name)?,
                        ret = return_type_annotation.to_js()?,
                    }
                }
                _ => {
                    formatdoc! {
                        r#"
                        auto ret = craby::{flat_name}::{fn_name}({fn_args});
                        return {ret};
                        "#,
                        flat_name = flat_case(mod_name),
                        fn_name = self.name,
                        fn_args = args.join(", "),
                        ret = return_type_annotation.to_js()?,
                    }
                }
            };

            (args_decls.join("\n"), invoke_stmts)
        } else {
            unreachable!()
        };

        let cxx_mod = cxx_mod_cls_name(mod_name);
        let args_count = self.args_count()?;

        let metadata = formatdoc! {
            r#"
            MethodMetadata{{{args_count}, &{cxx_mod}::{fn_name}}}
            "#,
            fn_name = self.name,
            cxx_mod = cxx_mod,
            args_count = args_count,
        };

        let impl_func = formatdoc! {
            r#"
            jsi::Value {cxx_mod}::{fn_name}(jsi::Runtime &rt,
                                            react::TurboModule &turboModule,
                                            const jsi::Value args[],
                                            size_t count) {{
              auto &thisModule = static_cast<{cxx_mod} &>(turboModule);
              auto callInvoker = thisModule.callInvoker_;

              try {{
                if (count != {args_count}) {{
                  throw jsi::JSError(rt, "Expected {args_count} argument{plural}");
                }}

            {args_decls}

            {invoke_stmts}

              }} catch (const jsi::JSError &err) {{
                throw err;
              }} catch (const std::exception &err) {{
                throw jsi::JSError(rt, craby::helpers::errorMessage(err));
              }}
            }}
            "#,
            fn_name = self.name,
            cxx_mod = cxx_mod,
            args_count = args_count,
            args_decls = indent_str(args_decls, 4),
            invoke_stmts = indent_str(invoke_stmts, 4),
            plural = if args_count == 1 { "" } else { "s" },
        };

        Ok(CxxMethod {
            name: self.name.clone(),
            metadata,
            impl_func,
        })
    }
}

pub mod template {
    use craby_common::utils::string::flat_case;
    use indoc::formatdoc;

    use crate::{constants::cxx_mod_cls_name, utils::indent_str};

    pub fn cxx_mod(name: &String) -> String {
        let flat_name = flat_case(name);
        let cxx_mod = cxx_mod_cls_name(name);

        formatdoc! {
            r#"
            #include "{cxx_mod}.hpp"
            #include "cxx.h"
            #include "ffi.rs.h"

            using namespace facebook;

            namespace craby {{
            namespace {flat_name} {{

            {cxx_mod}::{cxx_mod}(
                std::shared_ptr<react::CallInvoker> jsInvoker)
                : TurboModule({cxx_mod}::kModuleName, jsInvoker) {{
              callInvoker_ = std::move(jsInvoker);
            {method_maps}
            }}

            }} // namespace {flat_name}
            }} // namespace craby"#,
            flat_name = flat_name,
            cxx_mod = cxx_mod,
            method_maps = indent_str("".to_string(), 2), // TODO
        }
    }

    pub fn cxx_mod_header(name: &String, turbo_module_name: &String) -> String {
        let flat_name = flat_case(name);
        let cxx_mod = cxx_mod_cls_name(name);

        formatdoc! {
            r#"
            #pragma once

            #include <thread>
            #include <ReactCommon/TurboModule.h>
            #include <react/bridging/Bridging.h>
            #include <jsi/jsi.h>

            #include "cxx.h"
            #include "ffi.rs.h"

            #include "bridging.hpp"
            #include "helpers.hpp"

            namespace craby {{
            namespace {flat_name} {{

            class JSI_EXPORT {cxx_mod} : public facebook::react::TurboModule {{
            public:
              static constexpr const char *kModuleName = "{turbo_module_name}";

              {cxx_mod}(std::shared_ptr<facebook::react::CallInvoker> jsInvoker);

            {method_defs}

            protected:
              std::shared_ptr<facebook::react::CallInvoker> callInvoker_;
            }};

            }} // namespace {flat_name}
            }} // namespace craby"#,
            flat_name = flat_name,
            cxx_mod = cxx_mod,
            turbo_module_name = turbo_module_name,
            method_defs = indent_str("".to_string(), 2), // TODO
        }
    }

    pub fn cxx_method_def(name: &String) -> String {
        formatdoc! {
            r#"
            static facebook::jsi::Value
            {name}(facebook::jsi::Runtime &rt,
                    facebook::react::TurboModule &turboModule,
                    const facebook::jsi::Value args[], size_t count)
            "#,
            name = name,
        }
    }

    pub fn cxx_arg_ref(idx: usize) -> String {
        format!("args[{}]", idx)
    }

    pub fn cxx_arg_var(idx: usize) -> String {
        format!("__arg{}", idx)
    }
}
