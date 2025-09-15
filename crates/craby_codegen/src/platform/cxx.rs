use craby_common::utils::string::{pascal_case, snake_case};
use indoc::formatdoc;

use crate::types::{
    schema::{FunctionSpec, TypeAnnotation},
    types::ToType,
};

use super::rust::{ToExternType, ToSig};

pub struct CxxBridge {
    pub extern_func: String,
    pub impl_func: String,
}

pub trait ToCxxBridge {
    /// Returns the CXX(FFI) function signature for the `FunctionSpec`.
    ///
    /// ```rust,ignore
    /// // extern function
    /// #[cxx_name = "myFunc"]
    /// fn myFunc(arg1: Foo, arg2: Bar) -> Baz;
    ///
    /// // impl function
    /// fn myFunc(arg1: Foo, arg2: Bar) -> Baz {
    ///     MyModule::my_func(arg1, arg2)
    /// }
    /// ```
    fn to_cxx_bridge(&self, mod_name: &String) -> Result<CxxBridge, anyhow::Error>;
}

impl ToCxxBridge for FunctionSpec {
    fn to_cxx_bridge(&self, mod_name: &String) -> Result<CxxBridge, anyhow::Error> {
        match &*self.type_annotation {
            TypeAnnotation::FunctionTypeAnnotation {
                return_type_annotation,
                params,
            } => {
                let ret_type = return_type_annotation.to_type()?.to_string();
                let ret_extern_type = return_type_annotation.to_extern_type()?.to_string();
                let params_sig = params
                    .iter()
                    .map(|p| p.to_sig())
                    .collect::<Result<Vec<_>, _>>()
                    .map(|p| p.join(", "))?;

                let impl_name = pascal_case(mod_name);
                let fn_name = snake_case(&self.name);
                let fn_args = params.iter().map(|p| p.name.clone()).collect::<Vec<_>>();

                // If the return type is `void`, return an empty tuple.
                // Otherwise, return the given return type.
                let ret_extern_annotation = if ret_extern_type == "()" {
                    String::new()
                } else {
                    format!(" -> {}", ret_extern_type)
                };

                let ret_annotation = if ret_type == "()" {
                    String::new()
                } else {
                    format!(" -> {}", ret_type)
                };

                let extern_func = formatdoc! {
                    r#"
                  #[cxx_name = "{orig_fn_name}"]
                  fn {fn_name}({params_sig}){ret};"#,
                    orig_fn_name = self.name,
                    fn_name = fn_name,
                    params_sig = params_sig,
                    ret = ret_extern_annotation,
                };

                let impl_func = formatdoc! {
                    r#"
                  fn {fn_name}({params_sig}){ret} {{
                      {impl_name}::{fn_name}({fn_args})
                  }}"#,
                    params_sig = params_sig,
                    ret = ret_annotation,
                    impl_name = impl_name,
                    fn_name = fn_name.to_string(),
                    fn_args = fn_args.join(", "),
                };

                Ok(CxxBridge {
                    extern_func,
                    impl_func,
                })
            }
            _ => unimplemented!("Unsupported type annotation for function: {}", self.name),
        }
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

    pub fn cxx_mod_header(name: &String) -> String {
        let flat_name = flat_case(name);
        let cxx_mod = cxx_mod_cls_name(name);

        formatdoc! {
            r#"
            #pragma once

            #include <ReactCommon/TurboModule.h>
            #include <jsi/jsi.h>

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
            turbo_module_name = "".to_string(), // TODO
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
}
