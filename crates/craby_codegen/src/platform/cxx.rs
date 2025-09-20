use craby_common::utils::string::flat_case;
use indoc::formatdoc;
use template::{cxx_arg_ref, cxx_arg_var};

use crate::{
    constants::cxx_mod_cls_name,
    types::schema::{FunctionSpec, TypeAnnotation},
    utils::indent_str,
};

#[derive(Debug)]
pub struct CxxFromJs {
    pub expr: String,
}

#[derive(Debug)]
pub struct CxxToJs {
    pub expr: String,
}

#[derive(Debug, Clone)]
pub struct CxxMethod {
    /// Method name
    pub name: String,
    /// TurboModule's method metadata
    ///
    /// ```cpp
    /// MethodMetadata{1, &CxxMyTestModule::myFunc}
    /// ```
    pub metadata: String,
    /// Cxx function implementation
    ///
    /// ```cpp
    /// jsi::Value CxxMyTestModule::myFunc(jsi::Runtime &rt,
    ///                                    react::TurboModule &turboModule,
    ///                                    const jsi::Value args[],
    ///                                    size_t count) {
    ///     // Implementation here
    /// }
    /// ```
    pub impl_func: String,
}

impl TypeAnnotation {
    pub fn as_cxx_type(&self, mod_name: &String) -> Result<String, anyhow::Error> {
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
            | TypeAnnotation::StringLiteralUnionTypeAnnotation { .. } => "rust::String".to_string(),

            // Array type
            TypeAnnotation::ArrayTypeAnnotation { element_type } => {
                format!("rust::Vec<{}>", element_type.as_cxx_type(mod_name)?)
            }

            // Enum
            TypeAnnotation::EnumDeclaration { name, .. } => {
                format!("craby::{}::{}", flat_case(mod_name), name)
            }

            // Type alias
            TypeAnnotation::TypeAliasTypeAnnotation { name } => {
                format!("craby::{}::{}", flat_case(mod_name), name)
            }

            // Nullable type
            TypeAnnotation::NullableTypeAnnotation { type_annotation } => {
                let cxx_struct = match &**type_annotation {
                    TypeAnnotation::BooleanTypeAnnotation => "NullableBoolean".to_string(),
                    TypeAnnotation::NumberTypeAnnotation
                    | TypeAnnotation::FloatTypeAnnotation
                    | TypeAnnotation::DoubleTypeAnnotation
                    | TypeAnnotation::Int32TypeAnnotation
                    | TypeAnnotation::NumberLiteralTypeAnnotation { .. } => {
                        "NullableNumber".to_string()
                    }
                    TypeAnnotation::StringTypeAnnotation
                    | TypeAnnotation::StringLiteralTypeAnnotation { .. }
                    | TypeAnnotation::StringLiteralUnionTypeAnnotation { .. } => {
                        "NullableString".to_string()
                    }
                    TypeAnnotation::TypeAliasTypeAnnotation { name } => format!("Nullable{}", name),
                    TypeAnnotation::EnumDeclaration { name, .. } => format!("Nullable{}", name),
                    TypeAnnotation::ArrayTypeAnnotation { element_type } => match &**element_type {
                        TypeAnnotation::BooleanTypeAnnotation => "NullableBooleanArray".to_string(),
                        TypeAnnotation::NumberTypeAnnotation
                        | TypeAnnotation::FloatTypeAnnotation
                        | TypeAnnotation::DoubleTypeAnnotation
                        | TypeAnnotation::Int32TypeAnnotation
                        | TypeAnnotation::NumberLiteralTypeAnnotation { .. } => {
                            "NullableNumberArray".to_string()
                        }
                        TypeAnnotation::StringTypeAnnotation
                        | TypeAnnotation::StringLiteralTypeAnnotation { .. }
                        | TypeAnnotation::StringLiteralUnionTypeAnnotation { .. } => {
                            "NullableStringArray".to_string()
                        }
                        TypeAnnotation::TypeAliasTypeAnnotation { name } => {
                            format!("Nullable{}Array", name)
                        }
                        TypeAnnotation::EnumDeclaration { name, .. } => {
                            format!("Nullable{}Array", name)
                        }
                        _ => {
                            return Err(anyhow::anyhow!(
                                "[as_cxx_type] Unsupported type annotation for nullable array type: {:?}",
                                element_type
                            ))
                        }
                    },
                    _ => {
                        return Err(anyhow::anyhow!(
                            "[as_cxx_type] Unsupported type annotation for nullable type: {:?}",
                            type_annotation
                        ))
                    }
                };

                format!("craby::{}::{}", flat_case(mod_name), cxx_struct)
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
                ))
            }

            // Unsupported types
            _ => {
                return Err(anyhow::anyhow!(
                    "[as_cxx_type] Unsupported type annotation: {:?}",
                    self
                ))
            }
        };

        Ok(cxx_type)
    }

    pub fn as_cxx_default_val(&self, mod_name: &String) -> Result<String, anyhow::Error> {
        let default_val = match self {
            // Boolean type
            TypeAnnotation::BooleanTypeAnnotation => "false".to_string(),

            // Number types
            TypeAnnotation::NumberTypeAnnotation
            | TypeAnnotation::FloatTypeAnnotation
            | TypeAnnotation::DoubleTypeAnnotation
            | TypeAnnotation::Int32TypeAnnotation
            | TypeAnnotation::NumberLiteralTypeAnnotation { .. } => "0.0".to_string(),

            // String types
            TypeAnnotation::StringTypeAnnotation
            | TypeAnnotation::StringLiteralTypeAnnotation { .. }
            | TypeAnnotation::StringLiteralUnionTypeAnnotation { .. } => {
                "rust::String()".to_string()
            }

            // Array type
            TypeAnnotation::ArrayTypeAnnotation { element_type } => {
                format!("rust::Vec<{}>()", element_type.as_cxx_type(mod_name)?)
            }

            // Enum
            TypeAnnotation::EnumDeclaration { members, .. } => {
                let enum_type = self.as_cxx_type(mod_name)?;
                let first_member = members
                    .as_ref()
                    .expect("enum should have at least one member")
                    .first()
                    .clone()
                    .expect("enum should have at least one member");

                format!("{}::{}", enum_type, first_member.name)
            }

            // Type alias
            TypeAnnotation::TypeAliasTypeAnnotation { .. } => {
                let cxx_type = self.as_cxx_type(mod_name)?;
                format!("{}{{}}", cxx_type)
            }

            // Nullable type
            TypeAnnotation::NullableTypeAnnotation { .. } => {
                let cxx_type = self.as_cxx_type(mod_name)?;
                let default_val = self.as_cxx_default_val(mod_name)?;

                formatdoc! {
                    r#"
                    {cxx_type} {{
                        val: {default_val},
                        null: true,
                    }}
                    "#,
                    cxx_type = cxx_type,
                    default_val = default_val,
                }
            }

            _ => {
                return Err(anyhow::anyhow!(
                    "[as_cxx_default_val] Unsupported type annotation: {:?}",
                    self
                ))
            }
        };

        Ok(default_val)
    }

    /// Returns the cxx `fromJs` for the `TypeAnnotation`.
    ///
    /// ```cpp
    /// facebook::react::bridging::fromJs<T>(rt, value, callInvoker)
    /// ```
    pub fn as_cxx_from_js(
        &self,
        mod_name: &String,
        ident: &String,
    ) -> Result<CxxFromJs, anyhow::Error> {
        let from_js_expr = match &*self {
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
            | TypeAnnotation::TypeAliasTypeAnnotation { .. }
            // Nullable type
            | TypeAnnotation::NullableTypeAnnotation { .. } => format!(
                "react::bridging::fromJs<{}>(rt, {}, callInvoker)",
                self.as_cxx_type(mod_name)?, ident
            ),
            _ => return Err(anyhow::anyhow!("[as_cxx_from_js] Unsupported type annotation: {:?}", self)),
        };

        Ok(CxxFromJs { expr: from_js_expr })
    }

    /// Returns the cxx `toJs` for the `TypeAnnotation`.
    ///
    /// ```cpp
    /// react::bridging::toJs(rt, value)
    /// ```
    pub fn as_cxx_to_js(&self, ident: &String) -> Result<CxxToJs, anyhow::Error> {
        let to_js_expr = match &*self {
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
            | TypeAnnotation::TypeAliasTypeAnnotation { .. }
            // Nullable type
            | TypeAnnotation::NullableTypeAnnotation { .. } => format!("react::bridging::toJs(rt, {})", ident),
            // Promise type
            TypeAnnotation::PromiseTypeAnnotation { .. } => format!("react::bridging::toJs(rt, {})", ident),
            _ => return Err(anyhow::anyhow!("[as_cxx_to_js] Unsupported type annotation: {:?}", self)),
        };

        Ok(CxxToJs { expr: to_js_expr })
    }
}

impl FunctionSpec {
    pub fn as_cxx_method(&self, mod_name: &String) -> Result<CxxMethod, anyhow::Error> {
        let (args_decls, invoke_stmts) = if let TypeAnnotation::FunctionTypeAnnotation {
            return_type_annotation,
            params,
        } = &*self.type_annotation
        {
            // ["arg0", "arg1", "arg2"]
            let mut args = vec![];
            // ["auto arg0 = facebook::react::bridging::fromJs<T>(rt, value, callInvoker)", "..."]
            let mut args_decls = vec![];

            for (idx, param) in params.iter().enumerate() {
                let arg_ref = cxx_arg_ref(idx);
                let arg_var = cxx_arg_var(idx);
                let from_js = param.type_annotation.as_cxx_from_js(mod_name, &arg_ref)?;
                args.push(arg_var.clone());
                args_decls.push(format!("auto {} = {};", arg_var, from_js.expr));
            }

            let invoke_stmts = match &**return_type_annotation {
                TypeAnnotation::PromiseTypeAnnotation { element_type } => {
                    let fn_args = args.join(", ");
                    let mut bind_args = vec!["promise".to_string()];
                    bind_args.extend(args);

                    // Create a promise object and invoke the FFI function in a separate thread
                    //
                    // ```cpp
                    // react::AsyncPromise<T> promise(rt, callInvoker);
                    //
                    // std::thread([promise, arg0, arg1, arg2]() mutable {{
                    //   try {{
                    //     auto ret = craby::mymodule::myFunc(arg0, arg1, arg2);
                    //     promise.resolve(ret);
                    //   }} catch (const jsi::JSError &err) {{
                    //     promise.reject(err.getMessage());
                    //   }} catch (const std::exception &err) {{
                    //     promise.reject(errorMessage(err));
                    //   }}
                    // }}).detach();
                    //
                    // return promise;
                    // ```
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
                            promise.reject(errorMessage(err));
                          }}
                        }}).detach();

                        return {ret};"#,
                        bind_args = bind_args.join(", "),
                        fn_name = self.name,
                        fn_args = fn_args,
                        flat_name = flat_case(mod_name),
                        ret_type = element_type.as_cxx_type(mod_name)?,
                        ret = return_type_annotation.as_cxx_to_js(&"promise".to_string())?.expr,
                    }
                }
                _ => {
                    // Invoke the FFI function synchronously and return the result
                    //
                    // ```cpp
                    // auto ret = craby::mymodule::myFunc(arg0, arg1, arg2);
                    // return ret;
                    // ```
                    formatdoc! {
                        r#"
                        auto ret = craby::{flat_name}::{fn_name}({fn_args});

                        return {ret};"#,
                        flat_name = flat_case(mod_name),
                        fn_name = self.name,
                        fn_args = args.join(", "),
                        ret = return_type_annotation.as_cxx_to_js(&"ret".to_string())?.expr,
                    }
                }
            };

            (args_decls.join("\n"), invoke_stmts)
        } else {
            unreachable!()
        };

        let cxx_mod = cxx_mod_cls_name(mod_name);
        let args_count = self.args_count()?;

        // ```cpp
        // MethodMetadata{{1, &CxxMyTestModule::myFunc}}
        // ```
        let metadata = formatdoc! {
            r#"
            MethodMetadata{{{args_count}, &{cxx_mod}::{fn_name}}}"#,
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
                if ({args_count} != count) {{
                  throw jsi::JSError(rt, "Expected {args_count} argument{plural}");
                }}

            {args_decls}
            {invoke_stmts}
              }} catch (const jsi::JSError &err) {{
                throw err;
              }} catch (const std::exception &err) {{
                throw jsi::JSError(rt, errorMessage(err));
              }}
            }}"#,
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

    use crate::{
        constants::cxx_mod_cls_name,
        types::{
            schema::{Alias, Enum, TypeAnnotation},
            types::CodegenResult,
        },
        utils::indent_str,
    };

    /// Returns the complete cxx TurboModule implementation source file.
    pub fn mod_cxx(codegen_res: &Vec<CodegenResult>) -> String {
        let mut headers = vec![];
        let mod_namespaces = codegen_res
            .iter()
            .map(|res| {
                let flat_name = flat_case(&res.module_name);
                let cxx_mod = cxx_mod_cls_name(&res.module_name);

                // Assign method metadata with function pointer to the TurboModule's method map
                //
                // ```cpp
                // methodMap_["multiply"] = MethodMetadata{1, &CxxMyTestModule::multiply};
                // ```
                let method_maps = res
                    .cxx_methods
                    .iter()
                    .map(|method| format!("methodMap_[\"{}\"] = {};", method.name, method.metadata))
                    .collect::<Vec<_>>()
                    .join("\n");

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
                let method_impls = res
                    .cxx_methods
                    .iter()
                    .map(|method| method.impl_func.clone())
                    .collect::<Vec<_>>()
                    .join("\n\n");

                headers.push(format!("#include \"{}.hpp\"", cxx_mod));

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
                // // Method implementations
                //
                // } // namespace mymodule
                // ```
                formatdoc! {
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
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

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
        formatdoc! {
            r#"
            {headers}

            #include <thread>
            #include <react/bridging/Bridging.h>

            #include "cxx.h"
            #include "ffi.rs.h"
            #include "bridging-generated.hpp"
            #include "utils.hpp"

            using namespace facebook;

            namespace craby {{
            {mod_namespaces}
            }} // namespace craby"#,
            headers = headers.join("\n"),
            mod_namespaces = mod_namespaces,
        }
    }

    /// Returns the complete cxx TurboModule definition header file.
    pub fn mod_cxx_h(codegen_res: &Vec<CodegenResult>) -> String {
        let mod_namespaces = codegen_res
            .iter()
            .map(|res| {
                let flat_name = flat_case(&res.module_name);
                let cxx_mod = cxx_mod_cls_name(&res.module_name);
                let method_defs = res
                    .cxx_methods
                    .iter()
                    .map(|method| cxx_method_def(&method.name))
                    .collect::<Vec<_>>()
                    .join("\n\n");

                formatdoc! {
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
                    turbo_module_name = res.module_name,
                    method_defs = indent_str(method_defs, 2),
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        formatdoc! {
            r#"
            #pragma once

            #include <memory>
            #include <ReactCommon/TurboModule.h>
            #include <jsi/jsi.h>

            namespace craby {{
            {mod_namespaces}
            }} // namespace craby"#,
            mod_namespaces = mod_namespaces,
        }
    }

    /// Returns the complete cxx JSI bridging header file.
    pub fn cxx_bridging_h(codegen_res: &Vec<CodegenResult>) -> String {
        let mut has_template = false;
        let mut bridging_templates = vec![];

        codegen_res.iter().for_each(|res| {
            has_template = has_template || !res.cxx_bridging_templates.is_empty();
            res.cxx_bridging_templates.iter().for_each(|template| {
                bridging_templates.push(template.clone());
            });
        });

        formatdoc! {
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
            bridging_templates = if has_template { format!("\n{}\n", bridging_templates.join("\n\n")) } else { "".to_string() },
        }
    }

    /// Returns the cxx JSI bridging template for the `Alias`.
    pub fn cxx_struct_bridging_template(
        mod_name: &String,
        name: &String,
        alias: &Alias,
    ) -> Result<String, anyhow::Error> {
        if alias.r#type != "ObjectTypeAnnotation" {
            return Err(anyhow::anyhow!("Alias type should be ObjectTypeAnnotation"));
        }

        let flat_name = flat_case(mod_name);
        let struct_namespace = format!("craby::{}::{}", flat_name, name);

        let mut get_props = vec![];
        let mut set_props = vec![];
        let mut from_js_stmts = vec![];
        let mut from_js_ident = vec![];
        let mut to_js_stmts = vec![];

        alias
            .properties
            .iter()
            .try_for_each(|prop| -> Result<(), anyhow::Error> {
                let ident = format!("obj${}", prop.name);
                let converted_ident = format!("_{}", ident);
                let from_js = prop.type_annotation.as_cxx_from_js(&mod_name, &ident)?;
                let to_js = prop
                    .type_annotation
                    .as_cxx_to_js(&format!("value.{}", prop.name))?;

                // ```cpp
                // auto obj$name = obj.getProperty(rt, "name");
                // ```
                let get_prop = format!("auto {} = obj.getProperty(rt, \"{}\");", ident, prop.name);

                // ```cpp
                // obj.setProperty(rt, "name", _obj$name);
                // ```
                let set_prop = format!(
                    "obj.setProperty(rt, \"{}\", {});",
                    prop.name, converted_ident
                );

                // ```cpp
                // auto _obj$name = react::bridging::fromJs<T>(rt, value.name, callInvoker);
                // ```
                let from_js_stmt = format!("auto {} = {};", converted_ident, from_js.expr);

                // ```cpp
                // auto _obj$name = react::bridging::toJs(rt, value.name);
                // ```
                let to_js_stmt = format!("auto {} = {};", converted_ident, to_js.expr);

                get_props.push(get_prop);
                from_js_stmts.push(from_js_stmt);
                from_js_ident.push(converted_ident);
                set_props.push(set_prop);
                to_js_stmts.push(to_js_stmt);

                Ok(())
            })?;

        let from_js_impl = formatdoc! {
            r#"
            auto obj = value.asObject(rt);
            {get_props}

            {from_js_stmts}

            {struct_namespace} ret = {{
            {from_js_ident}
            }};

            return ret;"#,
            struct_namespace = struct_namespace,
            get_props = get_props.join("\n"),
            from_js_stmts = from_js_stmts.join("\n"),
            from_js_ident = indent_str(from_js_ident.join(",\n"), 2),
        };

        let to_js_impl = formatdoc! {
            r#"
            jsi::Object obj = jsi::Object(rt);
            {to_js_stmts}

            {set_props}

            return jsi::Value(rt, obj);"#,
            to_js_stmts = to_js_stmts.join("\n"),
            set_props = set_props.join("\n"),
        };

        let template = cxx_bridging_template(&struct_namespace, from_js_impl, to_js_impl);

        Ok(template)
    }

    /// Returns the cxx JSI bridging template for the `Enum`.
    pub fn cxx_enum_bridging_template(
        mod_name: &String,
        name: &String,
        enum_spec: &Enum,
    ) -> Result<String, anyhow::Error> {
        if enum_spec.r#type != "EnumDeclarationWithMembers" {
            return Err(anyhow::anyhow!(
                "Enum type should be EnumDeclarationWithMembers"
            ));
        }

        if !(enum_spec.member_type == "StringTypeAnnotation"
            || enum_spec.member_type == "NumberTypeAnnotation")
        {
            return Err(anyhow::anyhow!(
                "Enum member type should be StringTypeAnnotation or NumberTypeAnnotation: {}",
                name
            ));
        }

        if enum_spec.members.is_none() {
            return Err(anyhow::anyhow!("Enum members are required: {}", name));
        }

        let flat_name = flat_case(mod_name);
        let enum_namespace = format!("craby::{}::{}", flat_name, name);
        let as_raw = match enum_spec.member_type.as_str() {
            "StringTypeAnnotation" => "value.asString(rt).utf8(rt)",
            "NumberTypeAnnotation" => "value.asNumber()",
            _ => unreachable!(),
        };

        let raw_member = |value: &String| -> String {
            match enum_spec.member_type.as_str() {
                // "value"
                "StringTypeAnnotation" => format!("\"{}\"", value),
                // 123
                "NumberTypeAnnotation" => value.clone(),
                _ => unreachable!(),
            }
        };

        let mut from_js_conds = vec![];
        let mut to_js_conds = vec![];

        enum_spec
            .members
            .as_ref()
            .unwrap()
            .iter()
            .enumerate()
            .try_for_each(|(idx, member)| -> Result<(), anyhow::Error> {
                let enum_namespace = format!("{}::{}", enum_namespace, member.name);
                let raw_member = raw_member(&member.value.value);

                let from_js_cond = if idx == 0 {
                    // ```cpp
                    // if (raw == "value") {
                    //   return craby::mymodule::MyEnum::Value;
                    // }
                    // ```
                    formatdoc! {
                        r#"
                        if (raw == {raw_member}) {{
                          return {enum_namespace};
                        }}"#,
                        raw_member = raw_member,
                        enum_namespace = enum_namespace,
                    }
                } else {
                    // ```cpp
                    // else if (raw == "value2") {
                    //   return craby::mymodule::MyEnum::Value2;
                    // }
                    // ```
                    formatdoc! {
                        r#"
                        else if (raw == {raw_member}) {{
                          return {enum_namespace};
                        }}"#,
                        raw_member = raw_member,
                        enum_namespace = enum_namespace,
                    }
                };

                // ```cpp
                // case craby::mymodule::MyEnum::Value:
                //   return react::bridging::toJs(rt, "value");
                // ```
                let to_js_cond = formatdoc! {
                    r#"
                    case {enum_namespace}:
                      return react::bridging::toJs(rt, {raw_member});"#,
                    enum_namespace = enum_namespace,
                    raw_member = raw_member,
                };

                from_js_conds.push(from_js_cond);
                to_js_conds.push(to_js_cond);

                Ok(())
            })?;

        // ```cpp
        // else {
        //   throw jsi::JSError(rt, "Invalid enum value (MyEnum)");
        // }
        // ```
        from_js_conds.push(formatdoc! {
            r#"
            else {{
              throw jsi::JSError(rt, "Invalid enum value ({name})");
            }}"#,
            name = name,
        });

        // ```cpp
        // default:
        //   throw jsi::JSError(rt, "Invalid enum value (MyEnum)");
        // ```
        to_js_conds.push(formatdoc! {
            r#"
            default:
              throw jsi::JSError(rt, "Invalid enum value ({name})");"#,
            name = name,
        });

        // ```cpp
        // auto raw = value.asString(rt).utf8(rt);
        // if (raw == "value") {
        //   return craby::mymodule::MyEnum::Value;
        // } else if (raw == "value2") {
        //   return craby::mymodule::MyEnum::Value2;
        // } else {
        //   throw jsi::JSError(rt, "Invalid enum value (MyEnum)");
        // }
        // ```
        let from_js = formatdoc! {
            r#"
            auto raw = {as_raw};
            {from_js_conds}"#,
            as_raw = as_raw,
            from_js_conds = from_js_conds.join(" "),
        };

        // ```cpp
        // switch (value) {{
        //   case craby::mymodule::MyEnum::Value:
        //     return react::bridging::toJs(rt, "value");
        //   case craby::mymodule::MyEnum::Value2:
        //     return react::bridging::toJs(rt, "value2");
        //   default:
        //     throw jsi::JSError(rt, "Invalid enum value (MyEnum)");
        // }}
        // ```
        let to_js = formatdoc! {
            r#"
            switch (value) {{
            {to_js_conds}
            }}"#,
            to_js_conds = indent_str(to_js_conds.join("\n"), 2),
        };

        let template = cxx_bridging_template(&enum_namespace, from_js, to_js);

        Ok(template)
    }

    pub fn cxx_nullable_bridging_template(
        mod_name: &String,
        nullable_namespace: &String,
        type_annotation: &TypeAnnotation,
    ) -> Result<String, anyhow::Error> {
        let origin_namespace = type_annotation.as_cxx_type(mod_name)?;
        let default_value = type_annotation.as_cxx_default_val(mod_name)?;

        let from_js_impl = formatdoc! {
            r#"
            if (value.isNull()) {{
              return {nullable_namespace}{{true, {default_value}}};
            }}

            auto val = react::bridging::fromJs<{origin_namespace}>(rt, value, callInvoker);
            auto ret = {nullable_namespace}{{false, val}};

            return ret;"#,
            origin_namespace =  origin_namespace,
            nullable_namespace = nullable_namespace,
            default_value = default_value,
        };

        let to_js_impl = formatdoc! {
            r#"
            if (value.null) {{
              return jsi::Value::null();
            }}

            return react::bridging::toJs(rt, value.val);"#,
        };

        let template = cxx_bridging_template(&nullable_namespace, from_js_impl, to_js_impl);

        Ok(template)
    }

    /// Returns the cxx JSI bridging (`fromJs`, `toJs`) template.
    pub fn cxx_bridging_template(
        target_type: &String,
        from_js_impl: String,
        to_js_impl: String,
    ) -> String {
        formatdoc! {
            r#"
            template <>
            struct Bridging<{target_type}> {{
              static {target_type} fromJs(jsi::Runtime &rt, const jsi::Value& value, std::shared_ptr<CallInvoker> callInvoker) {{
            {from_js_impl}
              }}

              static jsi::Value toJs(jsi::Runtime &rt, {target_type} value) {{
            {to_js_impl}
              }}
            }};"#,
            target_type = target_type,
            from_js_impl = indent_str(from_js_impl, 4),
            to_js_impl = indent_str(to_js_impl, 4),
        }
    }

    /// Returns the cxx JSI method definition.
    ///
    /// ```cpp
    /// static facebook::jsi::Value
    /// myFunc(facebook::jsi::Runtime &rt,
    ///        facebook::react::TurboModule &turboModule,
    ///        const facebook::jsi::Value args[], size_t count);
    /// ```
    pub fn cxx_method_def(name: &String) -> String {
        formatdoc! {
            r#"
            static facebook::jsi::Value
            {name}(facebook::jsi::Runtime &rt,
                facebook::react::TurboModule &turboModule,
                const facebook::jsi::Value args[], size_t count);"#,
            name = name,
        }
    }

    pub fn cxx_arg_ref(idx: usize) -> String {
        format!("args[{}]", idx)
    }

    pub fn cxx_arg_var(idx: usize) -> String {
        format!("arg{}", idx)
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use crate::tests::load_schema_as_codegen_res;

    use super::*;

    #[test]
    fn test_mod_cxx() {
        let codegen_res = load_schema_as_codegen_res();
        let result = template::mod_cxx(&vec![codegen_res]);

        assert_snapshot!(result);
    }

    #[test]
    fn test_mod_cxx_h() {
        let codegen_res = load_schema_as_codegen_res();
        let result = template::mod_cxx_h(&vec![codegen_res]);

        assert_snapshot!(result);
    }

    #[test]
    fn test_cxx_bridging_h() {
        let codegen_res = load_schema_as_codegen_res();
        let result = template::cxx_bridging_h(&vec![codegen_res]);

        assert_snapshot!(result);
    }
}
