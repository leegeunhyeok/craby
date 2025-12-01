use std::collections::{btree_map::Entry as BTreeMapEntry, BTreeMap};

use craby_common::utils::string::camel_case;
use indoc::formatdoc;
use log::debug;
use template::{cxx_arg_ref, cxx_arg_var};

use crate::{
    common::IntoCode,
    constants::specs::RESERVED_ARG_NAME_MODULE,
    parser::types::{EnumTypeAnnotation, Method, ObjectTypeAnnotation, TypeAnnotation},
    platform::cxx::template::CxxBridgingTemplate,
    types::{CxxModuleName, CxxNamespace, Schema},
    utils::{calc_deps_order, indent_str},
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
    /// Converts TypeAnnotation to C++ type representation.
    ///
    /// # Generated Code Examples
    ///
    /// ```cpp
    /// bool                          // Boolean
    /// double                        // Number
    /// rust::Str                     // String (arguments)
    /// rust::String                  // String
    /// rust::Vec<double>             // Array<Number>
    /// craby::mymodule::bridging::MyEnum       // Enum
    /// craby::mymodule::bridging::MyStruct     // Object
    /// craby::mymodule::bridging::NullableNumber  // Nullable<Number>
    /// ```
    pub fn as_cxx_type(&self, cxx_ns: &CxxNamespace) -> Result<String, anyhow::Error> {
        let cxx_type = match self {
            TypeAnnotation::Void => "void".to_string(),
            TypeAnnotation::Boolean => "bool".to_string(),
            TypeAnnotation::Number => "double".to_string(),
            TypeAnnotation::String => "rust::String".to_string(),
            TypeAnnotation::ArrayBuffer => "rust::Vec<uint8_t>".to_string(),
            TypeAnnotation::Array(element_type) => {
                format!("rust::Vec<{}>", element_type.as_cxx_type(cxx_ns)?)
            }
            TypeAnnotation::Enum(EnumTypeAnnotation { name, .. }) => {
                format!("{cxx_ns}::bridging::{name}")
            }
            TypeAnnotation::Object(ObjectTypeAnnotation { name, .. }) => {
                format!("{cxx_ns}::bridging::{name}")
            }
            TypeAnnotation::Nullable(type_annotation) => {
                let cxx_struct = match &**type_annotation {
                    TypeAnnotation::Boolean => "NullableBoolean".to_string(),
                    TypeAnnotation::Number => "NullableNumber".to_string(),
                    TypeAnnotation::String => "NullableString".to_string(),
                    TypeAnnotation::Void => "NullableVoid".to_string(), 
                    TypeAnnotation::Object(ObjectTypeAnnotation { name, .. }) => format!("Nullable{}", name),
                    TypeAnnotation::Enum(EnumTypeAnnotation { name, .. }) => format!("Nullable{}", name),
                    TypeAnnotation::ArrayBuffer => "NullableArrayBuffer".to_string(),
                    TypeAnnotation::Array(element_type) => match &**element_type {
                        TypeAnnotation::Boolean => "NullableBooleanArray".to_string(),
                        TypeAnnotation::Number=> {
                            "NullableNumberArray".to_string()
                        }
                        TypeAnnotation::String => {
                            "NullableStringArray".to_string()
                        }
                        TypeAnnotation::Object(ObjectTypeAnnotation { name, .. }) => {
                            format!("Nullable{name}Array")
                        }
                        TypeAnnotation::Enum(EnumTypeAnnotation { name, .. }) => {
                            format!("Nullable{name}Array")
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

                format!("{cxx_ns}::bridging::{cxx_struct}")
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "[as_cxx_type] Unsupported type annotation: {:?}",
                    self
                ))
            }
        };

        Ok(cxx_type)
    }

    /// Generates default value for C++ types.
    ///
    /// # Generated Code Examples
    ///
    /// ```cpp
    /// false                                 // Boolean
    /// 0.0                                   // Number
    /// rust::String()                        // String
    /// rust::Vec<double>()                   // Array<Number>
    /// MyEnum::FirstMember                   // Enum
    /// craby::mymodule::bridging::MyStruct{} // Object
    /// ```
    pub fn as_cxx_default_val(&self, cxx_ns: &CxxNamespace) -> Result<String, anyhow::Error> {
        let default_val = match self {
            TypeAnnotation::Boolean => "false".to_string(),
            TypeAnnotation::Number => "0.0".to_string(),
            TypeAnnotation::String => "rust::String()".to_string(),
            TypeAnnotation::ArrayBuffer => "rust::Vec<uint8_t>()".to_string(),
            TypeAnnotation::Array(element_type) => {
                format!("rust::Vec<{}>()", element_type.as_cxx_type(cxx_ns)?)
            }
            TypeAnnotation::Enum(EnumTypeAnnotation { members, .. }) => {
                let enum_type = self.as_cxx_type(cxx_ns)?;
                let first_member = members
                    .first()
                    .ok_or(anyhow::anyhow!("Enum should have at least one member"))?;

                format!("{enum_type}::{}", first_member.name)
            }
            TypeAnnotation::Object(..) => {
                let cxx_type = self.as_cxx_type(cxx_ns)?;
                format!("{cxx_type}{{}}")
            }
            TypeAnnotation::Nullable(..) => {
                let cxx_type = self.as_cxx_type(cxx_ns)?;
                let default_val = self.as_cxx_default_val(cxx_ns)?;
                formatdoc! {
                    r#"
                    {cxx_type} {{
                        val: {default_val},
                        null: true,
                    }}
                    "#,
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
        cxx_ns: &CxxNamespace,
        ident: &str,
    ) -> Result<CxxFromJs, anyhow::Error> {
        let from_js_expr = match self {
            TypeAnnotation::Boolean
            | TypeAnnotation::Number
            | TypeAnnotation::String
            | TypeAnnotation::ArrayBuffer
            | TypeAnnotation::Array(..)
            | TypeAnnotation::Enum(..)
            | TypeAnnotation::Object(..)
            | TypeAnnotation::Nullable(..) => format!(
                "react::bridging::fromJs<{}>(rt, {ident}, callInvoker)",
                self.as_cxx_type(cxx_ns)?,
            ),
            _ => {
                return Err(anyhow::anyhow!(
                    "[as_cxx_from_js] Unsupported type annotation: {:?}",
                    self
                ))
            }
        };

        Ok(CxxFromJs { expr: from_js_expr })
    }

    /// Returns the cxx `toJs` for the `TypeAnnotation`.
    ///
    /// ```cpp
    /// react::bridging::toJs(rt, value)
    /// ```
    pub fn as_cxx_to_js(&self, ident: &str) -> Result<CxxToJs, anyhow::Error> {
        let to_js_expr = match self {
            TypeAnnotation::Boolean
            | TypeAnnotation::Number
            | TypeAnnotation::String
            | TypeAnnotation::ArrayBuffer
            | TypeAnnotation::Array(..)
            | TypeAnnotation::Enum(..)
            | TypeAnnotation::Object(..)
            | TypeAnnotation::Nullable(..) => format!("react::bridging::toJs(rt, {})", ident),
            TypeAnnotation::Promise(..) => {
                format!("react::bridging::toJs(rt, {})", ident)
            }
            TypeAnnotation::Void => "jsi::Value::undefined()".to_string(),
            _ => {
                return Err(anyhow::anyhow!(
                    "[as_cxx_to_js] Unsupported type annotation: {:?}",
                    self
                ))
            }
        };

        Ok(CxxToJs { expr: to_js_expr })
    }
}

impl Method {
    /// Converts schema Method to C++ TurboModule method implementation.
    ///
    /// # Generated Code
    ///
    /// ```cpp
    /// jsi::Value CxxMyTestModule::multiply(jsi::Runtime &rt,
    ///                                       react::TurboModule &turboModule,
    ///                                       const jsi::Value args[],
    ///                                       size_t count) {
    ///   auto &thisModule = static_cast<CxxMyTestModule &>(turboModule);
    ///   auto callInvoker = thisModule.callInvoker_;
    ///   auto it_ = thisModule.module_;
    ///
    ///   try {
    ///     if (2 != count) {
    ///       throw jsi::JSError(rt, "Expected 2 arguments");
    ///     }
    ///
    ///     auto arg0 = react::bridging::fromJs<double>(rt, args[0], callInvoker);
    ///     auto arg1 = react::bridging::fromJs<double>(rt, args[1], callInvoker);
    ///     auto ret = craby::calculator::bridging::multiply(*it_, arg0, arg1);
    ///
    ///     return react::bridging::toJs(rt, ret);
    ///   } catch (const jsi::JSError &err) {
    ///     throw err;
    ///   } catch (const std::exception &err) {
    ///     throw jsi::JSError(rt, craby::calculator::utils::errorMessage(err));
    ///   }
    /// }
    /// ```
    pub fn as_cxx_method(
        &self,
        cxx_ns: &CxxNamespace,
        cxx_mod: &CxxModuleName,
    ) -> Result<CxxMethod, anyhow::Error> {
        let fn_name = camel_case(&self.name);
        // ["arg0", "arg1", "arg2"]
        let mut args = Vec::with_capacity(self.params.len() + 1);
        // ["auto arg0 = facebook::react::bridging::fromJs<T>(rt, value, callInvoker)", "..."]
        let mut args_decls = Vec::with_capacity(self.params.len());

        for (idx, param) in self.params.iter().enumerate() {
            let arg_ref = cxx_arg_ref(idx);
            let arg_var = cxx_arg_var(idx);

            // `rust::Str` holds a reference to `std::string`.
            // To avoid dangling pointers, the converted `std::string` is retained within the scope for the lifetime of the reference.
            let from_js = if let TypeAnnotation::String = &param.type_annotation {
                // Capture the converted `std::string` within the scope of the reference
                let str_var = format!("{arg_var}$raw");
                args_decls.push(format!("auto {str_var} = {arg_ref}.asString(rt).utf8(rt);",));

                // Convert the `std::string` to `rust::Str`
                format!("rust::Str({str_var}.data(), {str_var}.size())")
            } else {
                param.type_annotation.as_cxx_from_js(cxx_ns, &arg_ref)?.expr
            };
            args.push(arg_var.clone());
            args_decls.push(format!("auto {arg_var} = {from_js};"));
        }

        let invoke_stmts = match &self.ret_type {
            TypeAnnotation::Promise(resolve_type) => {
                let mut bind_args = Vec::with_capacity(args.len() + 2);
                bind_args.push(RESERVED_ARG_NAME_MODULE.to_string());
                bind_args.push("promise".to_string());
                bind_args.extend(args.clone());

                args.insert(0, format!("*{}", RESERVED_ARG_NAME_MODULE));
                let fn_args = args.join(", ");

                let ret_stmts = if let TypeAnnotation::Void = &**resolve_type {
                    formatdoc! {
                        r#"
                        {cxx_ns}::bridging::{fn_name}({fn_args});
                        promise.resolve(std::monostate{{}});
                        "#,
                    }
                } else {
                    formatdoc! {
                        r#"
                        auto ret = {cxx_ns}::bridging::{fn_name}({fn_args});
                        promise.resolve(ret);
                        "#,
                    }
                };

                let bind_args = bind_args.join(", ");
                let ret_stmts = indent_str(&ret_stmts, 4);
                let ret_type = if let TypeAnnotation::Void = &**resolve_type {
                    "std::monostate".to_string()
                } else {
                    resolve_type.as_cxx_type(cxx_ns)?
                };
                let ret = self.ret_type.as_cxx_to_js("promise")?.expr;

                // Create a promise object and invoke the FFI function in a separate thread
                formatdoc! {
                    r#"
                    react::AsyncPromise<{ret_type}> promise(rt, callInvoker);

                    thisModule.threadPool_->enqueue([{bind_args}]() mutable {{
                      try {{
                    {ret_stmts}
                      }} catch (const jsi::JSError &err) {{
                        promise.reject(err.getMessage());
                      }} catch (const std::exception &err) {{
                        promise.reject({cxx_ns}::utils::errorMessage(err));
                      }}
                    }});

                    return {ret};"#,
                }
            }
            _ => {
                // Invoke the FFI function synchronously and return the result
                //
                // ```cpp
                // auto ret = craby::mymodule::bridging::myFunc(arg0, arg1, arg2);
                // return ret;
                // ```
                args.insert(0, format!("*{RESERVED_ARG_NAME_MODULE}"));
                let fn_args = args.join(", ");
                let ret_stmts = if let TypeAnnotation::Void = &self.ret_type {
                    format!("{cxx_ns}::bridging::{fn_name}({fn_args});")
                } else {
                    format!("auto ret = {cxx_ns}::bridging::{fn_name}({fn_args});")
                };

                formatdoc! {
                    r#"
                    {ret_stmts}

                    return {to_js};"#,
                    to_js = self.ret_type.as_cxx_to_js("ret")?.expr,
                }
            }
        };

        let args_decls = args_decls.join("\n");
        let args_count = self.params.len();

        // ```cpp
        // MethodMetadata{{1, &CxxMyTestModule::myFunc}}
        // ```
        let metadata = formatdoc! {
            r#"
            MethodMetadata{{{args_count}, &{cxx_mod}::{fn_name}}}"#,
        };

        let invoke_stmts = indent_str([args_decls, invoke_stmts].join("\n").trim(), 4);
        let impl_func = formatdoc! {
            r#"
            jsi::Value {cxx_mod}::{fn_name}(jsi::Runtime &rt,
                                            react::TurboModule &turboModule,
                                            const jsi::Value args[],
                                            size_t count) {{
              auto &thisModule = static_cast<{cxx_mod} &>(turboModule);
              auto callInvoker = thisModule.callInvoker_;
              auto it_ = thisModule.module_;

              try {{
                if ({args_count} != count) {{
                  throw jsi::JSError(rt, "Expected {args_count} argument{plural}");
                }}

            {invoke_stmts}
              }} catch (const jsi::JSError &err) {{
                throw err;
              }} catch (const std::exception &err) {{
                throw jsi::JSError(rt, {cxx_ns}::utils::errorMessage(err));
              }}
            }}"#,
            plural = if args_count > 1 { "s" } else { "" },
        };

        Ok(CxxMethod {
            name: self.name.clone(),
            metadata,
            impl_func,
        })
    }
}

impl Schema {
    /// Generates C++ bridging templates for custom types (structs, enums, nullables).
    ///
    /// # Generated Code
    ///
    /// ```cpp
    /// template <>
    /// struct Bridging<craby::mymodule::bridging::MyStruct> {
    ///   static craby::mymodule::bridging::MyStruct fromJs(jsi::Runtime &rt, const jsi::Value& value, std::shared_ptr<CallInvoker> callInvoker) {
    ///     auto obj = value.asObject(rt);
    ///     auto obj$foo = obj.getProperty(rt, "foo");
    ///     auto _obj$foo = react::bridging::fromJs<rust::String>(rt, obj$foo, callInvoker);
    ///
    ///     craby::mymodule::bridging::MyStruct ret = {
    ///       _obj$foo
    ///     };
    ///
    ///     return ret;
    ///   }
    ///
    ///   static jsi::Value toJs(jsi::Runtime &rt, craby::mymodule::bridging::MyStruct value) {
    ///     jsi::Object obj = jsi::Object(rt);
    ///     auto _obj$foo = react::bridging::toJs(rt, value.foo);
    ///     obj.setProperty(rt, "foo", _obj$foo);
    ///
    ///     return jsi::Value(rt, obj);
    ///   }
    /// };
    /// ```
    pub fn as_cxx_bridging_templates(
        &self,
        project_name: &str,
    ) -> Result<Vec<String>, anyhow::Error> {
        let cxx_ns = CxxNamespace::from(project_name);
        let mut bridging_templates = BTreeMap::new();
        let mut enum_bridging_templates = BTreeMap::new();
        let mut nullable_bridging_templates = self.collect_nullable_types(project_name)?;

        for type_annotation in &self.aliases {
            let alias_spec = type_annotation.as_object().unwrap();
            bridging_templates.insert(
                alias_spec.name.clone(),
                CxxBridgingTemplate::try_into_struct_template(&cxx_ns, alias_spec)?.into_code(),
            );
        }

        for type_annotation in &self.enums {
            let enum_spec = type_annotation.as_enum().unwrap();
            enum_bridging_templates.insert(
                enum_spec.name.clone(),
                CxxBridgingTemplate::try_into_enum_template(&cxx_ns, enum_spec)?.into_code(),
            );
        }

        // C++ Templates are should be sorted in the order of their dependencies
        let ord = calc_deps_order(self)?;
        let mut ordered_templates = vec![];
        debug!("CXX Bridging templates dependencies order: {:?}", ord);

        ordered_templates.extend(enum_bridging_templates.into_values());

        ord.iter().for_each(|name| {
            if let Some(template) = bridging_templates.remove(name) {
                ordered_templates.push(template);
            }

            if let Some(template) =
                nullable_bridging_templates.remove(&format!("{cxx_ns}::bridging::{name}"))
            {
                ordered_templates.push(template);
            }
        });

        ordered_templates.extend(bridging_templates.into_values());
        ordered_templates.extend(nullable_bridging_templates.into_values());

        Ok(ordered_templates)
    }

    /// Collects all nullable types from schema to generate bridging templates.
    ///
    /// # Generated Code
    ///
    /// ```cpp
    /// template <>
    /// struct Bridging<craby::mymodule::bridging::NullableNumber> {
    ///   static craby::mymodule::bridging::NullableNumber fromJs(jsi::Runtime &rt, const jsi::Value& value, std::shared_ptr<CallInvoker> callInvoker) {
    ///     if (value.isNull()) {
    ///       return craby::mymodule::bridging::NullableNumber{true, 0.0};
    ///     }
    ///
    ///     auto val = react::bridging::fromJs<double>(rt, value, callInvoker);
    ///     auto ret = craby::mymodule::bridging::NullableNumber{false, val};
    ///
    ///     return ret;
    ///   }
    ///
    ///   static jsi::Value toJs(jsi::Runtime &rt, craby::mymodule::bridging::NullableNumber value) {
    ///     if (value.null) {
    ///       return jsi::Value::null();
    ///     }
    ///
    ///     return react::bridging::toJs(rt, value.val);
    ///   }
    /// };
    /// ```
    pub fn collect_nullable_types(
        &self,
        project_name: &str,
    ) -> Result<BTreeMap<String, String>, anyhow::Error> {
        let cxx_ns = CxxNamespace::from(project_name);
        let mut templates = BTreeMap::new();

        for method in &self.methods {
            for param in &method.params {
                if let nullable_type @ TypeAnnotation::Nullable(inner_type_annotation) =
                    &param.type_annotation
                {
                    let key = nullable_type.as_cxx_type(&cxx_ns)?;
                    if let BTreeMapEntry::Vacant(e) = templates.entry(key) {
                        let bridging_template = CxxBridgingTemplate::try_into_nullable_template(
                            &cxx_ns,
                            nullable_type,
                            inner_type_annotation,
                        )?
                        .into_code();
                        e.insert(bridging_template);
                    }
                }
            }

            if let nullable_type @ TypeAnnotation::Nullable(inner_type_annotation) =
                &method.ret_type
            {
                let key = nullable_type.as_cxx_type(&cxx_ns)?;
                if let BTreeMapEntry::Vacant(e) = templates.entry(key) {
                    let bridging_template = CxxBridgingTemplate::try_into_nullable_template(
                        &cxx_ns,
                        nullable_type,
                        inner_type_annotation,
                    )?
                    .into_code();
                    e.insert(bridging_template);
                }
            }
        }

        for type_annotation in &self.aliases {
            for prop in &type_annotation.as_object().unwrap().props {
                if let nullable_type @ TypeAnnotation::Nullable(inner_type_annotation) =
                    &prop.type_annotation
                {
                    let key = nullable_type.as_cxx_type(&cxx_ns)?;
                    if let BTreeMapEntry::Vacant(e) = templates.entry(key) {
                        let bridging_template = CxxBridgingTemplate::try_into_nullable_template(
                            &cxx_ns,
                            nullable_type,
                            inner_type_annotation,
                        )?
                        .into_code();
                        e.insert(bridging_template);
                    }
                }
            }
        }

        Ok(templates)
    }
}

pub mod template {
    use craby_common::utils::string::{camel_case, snake_case};
    use indoc::formatdoc;

    use crate::{
        common::IntoCode,
        parser::types::{
            EnumMemberValue as ParserEnumMemberValue, EnumTypeAnnotation, ObjectTypeAnnotation,
            TypeAnnotation,
        },
        types::CxxNamespace,
        utils::indent_str,
    };

    pub struct CxxBridgingTemplate {
        pub namespace: String,
        pub from_js: String,
        pub to_js: String,
    }

    impl IntoCode for CxxBridgingTemplate {
        fn into_code(self) -> String {
            self.cxx_bridging_template()
        }
    }

    impl CxxBridgingTemplate {
        /// Generates a generic C++ JSI bridging template.
        ///
        /// # Generated Code
        ///
        /// ```cpp
        /// template <>
        /// struct Bridging<TargetType> {
        ///   static TargetType fromJs(jsi::Runtime &rt, const jsi::Value& value, std::shared_ptr<CallInvoker> callInvoker) {
        ///     // fromJs implementation
        ///   }
        ///
        ///   static jsi::Value toJs(jsi::Runtime &rt, TargetType value) {
        ///     // toJs implementation
        ///   }
        /// };
        /// ```
        fn cxx_bridging_template(&self) -> String {
            let from_js_impl = indent_str(&self.from_js, 4);
            let to_js_impl = indent_str(&self.to_js, 4);
            formatdoc! {
                r#"
                template <>
                struct Bridging<{namespace}> {{
                  static {namespace} fromJs(jsi::Runtime &rt, const jsi::Value& value, std::shared_ptr<CallInvoker> callInvoker) {{
                {from_js_impl}
                  }}
    
                  static jsi::Value toJs(jsi::Runtime &rt, {namespace} value) {{
                {to_js_impl}
                  }}
                }};"#,
                namespace = self.namespace,
            }
        }

        /// Generates C++ bridging template for struct/object types.
        ///
        /// # Generated Code
        ///
        /// ```cpp
        /// template <>
        /// struct Bridging<craby::mymodule::bridging::MyStruct> {
        ///   static craby::mymodule::bridging::MyStruct fromJs(jsi::Runtime &rt, const jsi::Value& value, std::shared_ptr<CallInvoker> callInvoker) {
        ///     auto obj = value.asObject(rt);
        ///     auto obj$foo = obj.getProperty(rt, "foo");
        ///
        ///     auto _obj$foo = react::bridging::fromJs<rust::String>(rt, value.foo, callInvoker);
        ///
        ///     craby::mymodule::bridging::MyStruct ret = {
        ///       _obj$foo
        ///     };
        ///
        ///     return ret;
        ///   }
        ///
        ///   static jsi::Value toJs(jsi::Runtime &rt, craby::mymodule::bridging::MyStruct value) {
        ///     jsi::Object obj = jsi::Object(rt);
        ///     auto _obj$foo = react::bridging::toJs(rt, value.foo);
        ///
        ///     obj.setProperty(rt, "foo", _obj$foo);
        ///
        ///     return jsi::Value(rt, obj);
        ///   }
        /// };
        /// ```
        pub fn try_into_struct_template(
            cxx_ns: &CxxNamespace,
            obj: &ObjectTypeAnnotation,
        ) -> Result<CxxBridgingTemplate, anyhow::Error> {
            let struct_namespace = format!("{cxx_ns}::bridging::{}", obj.name);
            let mut get_props = vec![];
            let mut set_props = vec![];
            let mut from_js_stmts = vec![];
            let mut from_js_ident = vec![];
            let mut to_js_stmts = vec![];

            for prop in &obj.props {
                let ident = format!("obj${}", camel_case(&prop.name));
                let converted_ident = format!("_{}", ident);
                let from_js = prop.type_annotation.as_cxx_from_js(cxx_ns, &ident)?;
                let to_js = prop
                    .type_annotation
                    .as_cxx_to_js(&format!("value.{}", snake_case(&prop.name)))?;

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
            }

            let get_props = get_props.join("\n");
            let from_js_stmts = from_js_stmts.join("\n");
            let from_js_ident = indent_str(&from_js_ident.join(",\n"), 2);
            let from_js_impl = formatdoc! {
                r#"
                auto obj = value.asObject(rt);
                {get_props}
    
                {from_js_stmts}
    
                {struct_namespace} ret = {{
                {from_js_ident}
                }};
    
                return ret;"#,
            };

            let to_js_stmts = to_js_stmts.join("\n");
            let set_props = set_props.join("\n");
            let to_js_impl = formatdoc! {
                r#"
                jsi::Object obj = jsi::Object(rt);
                {to_js_stmts}
    
                {set_props}
    
                return jsi::Value(rt, obj);"#,
            };

            Ok(CxxBridgingTemplate {
                namespace: struct_namespace,
                from_js: from_js_impl,
                to_js: to_js_impl,
            })
        }

        /// Generates C++ bridging template for enum types.
        ///
        /// # Generated Code
        ///
        /// ```cpp
        /// template <>
        /// struct Bridging<craby::mymodule::bridging::MyEnum> {
        ///   static craby::mymodule::bridging::MyEnum fromJs(jsi::Runtime &rt, const jsi::Value& value, std::shared_ptr<CallInvoker> callInvoker) {
        ///     auto raw = value.asString(rt).utf8(rt);
        ///     if (raw == "foo") {
        ///       return craby::mymodule::bridging::MyEnum::Foo;
        ///     } else if (raw == "bar") {
        ///       return craby::mymodule::bridging::MyEnum::Bar;
        ///     } else {
        ///       throw jsi::JSError(rt, "Invalid enum value (MyEnum)");
        ///     }
        ///   }
        ///
        ///   static jsi::Value toJs(jsi::Runtime &rt, craby::mymodule::bridging::MyEnum value) {
        ///     switch (value) {
        ///       case craby::mymodule::bridging::MyEnum::Foo:
        ///         return react::bridging::toJs(rt, "foo");
        ///       case craby::mymodule::bridging::MyEnum::Bar:
        ///         return react::bridging::toJs(rt, "bar");
        ///       default:
        ///         throw jsi::JSError(rt, "Invalid enum value (MyEnum)");
        ///     }
        ///   }
        /// };
        /// ```
        pub fn try_into_enum_template(
            cxx_ns: &CxxNamespace,
            enum_spec: &EnumTypeAnnotation,
        ) -> Result<CxxBridgingTemplate, anyhow::Error> {
            let enum_namespace = format!("{cxx_ns}::bridging::{}", enum_spec.name);
            let is_str = match enum_spec.members.first().unwrap().value {
                ParserEnumMemberValue::String { .. } => true,
                ParserEnumMemberValue::Number { .. } => false,
            };

            let as_raw = if is_str {
                "value.asString(rt).utf8(rt)"
            } else {
                "value.asNumber()"
            };

            let to_raw_member = |value: &String| -> String {
                if is_str {
                    // "value"
                    format!("\"{value}\"")
                } else {
                    // 123
                    value.clone()
                }
            };

            let mut from_js_conds = vec![];
            let mut to_js_conds = vec![];

            enum_spec.members.iter().enumerate().try_for_each(
                |(idx, member)| -> Result<(), anyhow::Error> {
                    let enum_namespace = format!("{}::{}", enum_namespace, member.name);
                    let raw_member = match &member.value {
                        ParserEnumMemberValue::String(val) => to_raw_member(val),
                        ParserEnumMemberValue::Number(val) => to_raw_member(&val.to_string()),
                    };

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
                    };

                    from_js_conds.push(from_js_cond);
                    to_js_conds.push(to_js_cond);

                    Ok(())
                },
            )?;

            // ```cpp
            // else {
            //   throw jsi::JSError(rt, "Invalid enum value (MyEnum)");
            // }
            // ```
            from_js_conds.push(formatdoc! {
                r#"
                else {{
                  throw jsi::JSError(rt, "Invalid enum value ({enum_name})");
                }}"#,
                enum_name = enum_spec.name,
            });

            // ```cpp
            // default:
            //   throw jsi::JSError(rt, "Invalid enum value (MyEnum)");
            // ```
            to_js_conds.push(formatdoc! {
                r#"
                default:
                  throw jsi::JSError(rt, "Invalid enum value ({enum_name})");"#,
                enum_name = enum_spec.name,
            });

            let from_js_conds = from_js_conds.join(" ");
            let to_js_conds = indent_str(&to_js_conds.join("\n"), 2);

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
            let from_js_impl = formatdoc! {
                r#"
                auto raw = {as_raw};
                {from_js_conds}"#,
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
            let to_js_impl = formatdoc! {
                r#"
                switch (value) {{
                {to_js_conds}
                }}"#,
            };

            Ok(CxxBridgingTemplate {
                namespace: enum_namespace,
                from_js: from_js_impl,
                to_js: to_js_impl,
            })
        }

        /// Generates C++ bridging template for nullable types.
        ///
        /// # Generated Code
        ///
        /// ```cpp
        /// template <>
        /// struct Bridging<craby::mymodule::bridging::NullableNumber> {
        ///   static craby::mymodule::bridging::NullableNumber fromJs(jsi::Runtime &rt, const jsi::Value& value, std::shared_ptr<CallInvoker> callInvoker) {
        ///     if (value.isNull()) {
        ///       return craby::mymodule::bridging::NullableNumber{true, 0.0};
        ///     }
        ///
        ///     auto val = react::bridging::fromJs<double>(rt, value, callInvoker);
        ///     auto ret = craby::mymodule::bridging::NullableNumber{false, val};
        ///
        ///     return ret;
        ///   }
        ///
        ///   static jsi::Value toJs(jsi::Runtime &rt, craby::mymodule::bridging::NullableNumber value) {
        ///     if (value.null) {
        ///       return jsi::Value::null();
        ///     }
        ///
        ///     return react::bridging::toJs(rt, value.val);
        ///   }
        /// };
        /// ```
        pub fn try_into_nullable_template(
            cxx_ns: &CxxNamespace,
            nullable_type_annotation: &TypeAnnotation,
            type_annotation: &TypeAnnotation,
        ) -> Result<CxxBridgingTemplate, anyhow::Error> {
            let origin_namespace = type_annotation.as_cxx_type(cxx_ns)?;
            let default_value = type_annotation.as_cxx_default_val(cxx_ns)?;
            let nullable_type_namespace = nullable_type_annotation.as_cxx_type(cxx_ns)?;

            let from_js_impl = formatdoc! {
                r#"
                if (value.isNull()) {{
                  return {nullable_type_namespace}{{true, {default_value}}};
                }}

                auto val = react::bridging::fromJs<{origin_namespace}>(rt, value, callInvoker);
                auto ret = {nullable_type_namespace}{{false, val}};

                return ret;"#,
            };

            let to_js_impl = formatdoc! {
                r#"
                if (value.null) {{
                  return jsi::Value::null();
                }}

                return react::bridging::toJs(rt, value.val);"#,
            };

            Ok(CxxBridgingTemplate {
                namespace: nullable_type_namespace.clone(),
                from_js: from_js_impl,
                to_js: to_js_impl,
            })
        }
    }

    /// Generates C++ argument reference expression.
    ///
    /// # Generated Code
    ///
    /// ```cpp
    /// args[0]  // idx = 0
    /// args[1]  // idx = 1
    /// ```
    pub fn cxx_arg_ref(idx: usize) -> String {
        format!("args[{idx}]")
    }

    /// Generates C++ argument variable name.
    ///
    /// # Generated Code
    ///
    /// ```cpp
    /// arg0  // idx = 0
    /// arg1  // idx = 1
    /// ```
    pub fn cxx_arg_var(idx: usize) -> String {
        format!("arg{idx}")
    }
}
