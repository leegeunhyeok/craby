use std::collections::BTreeMap;

use indoc::formatdoc;
use log::debug;
use template::{cxx_arg_ref, cxx_arg_var};

use crate::{
    constants::{cxx_mod_cls_name, specs::RESERVED_ARG_NAME_ID},
    parser::types::{EnumTypeAnnotation, Method, ObjectTypeAnnotation, TypeAnnotation},
    platform::cxx::template::{
        cxx_enum_bridging_template, cxx_nullable_bridging_template, cxx_struct_bridging_template,
    },
    types::Schema,
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
    pub fn as_cxx_type(&self, mod_name: &String) -> Result<String, anyhow::Error> {
        let cxx_type = match self {
            TypeAnnotation::Boolean => "bool".to_string(),
            TypeAnnotation::Number => "double".to_string(),
            TypeAnnotation::String => "rust::String".to_string(),
            TypeAnnotation::Array(element_type) => {
                format!("rust::Vec<{}>", element_type.as_cxx_type(mod_name)?)
            }
            TypeAnnotation::Enum(EnumTypeAnnotation { name, .. }) => {
                format!("craby::bridging::{}", name)
            }
            TypeAnnotation::Object(ObjectTypeAnnotation { name, .. }) => {
                format!("craby::bridging::{}", name)
            }
            TypeAnnotation::Nullable(type_annotation) => {
                let cxx_struct = match &**type_annotation {
                    TypeAnnotation::Boolean => "NullableBoolean".to_string(),
                    TypeAnnotation::Number => "NullableNumber".to_string(),
                    TypeAnnotation::String => "NullableString".to_string(),
                    TypeAnnotation::Object(ObjectTypeAnnotation { name, .. }) => format!("Nullable{}", name),
                    TypeAnnotation::Enum(EnumTypeAnnotation { name, .. }) => format!("Nullable{}", name),
                    TypeAnnotation::Array(element_type) => match &**element_type {
                        TypeAnnotation::Boolean => "NullableBooleanArray".to_string(),
                        TypeAnnotation::Number=> {
                            "NullableNumberArray".to_string()
                        }
                        TypeAnnotation::String => {
                            "NullableStringArray".to_string()
                        }
                        TypeAnnotation::Object(ObjectTypeAnnotation { name, .. }) => {
                            format!("Nullable{}Array", name)
                        }
                        TypeAnnotation::Enum(EnumTypeAnnotation { name, .. }) => {
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

                format!("craby::bridging::{}", cxx_struct)
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

    pub fn as_cxx_default_val(&self, mod_name: &String) -> Result<String, anyhow::Error> {
        let default_val = match self {
            TypeAnnotation::Boolean => "false".to_string(),
            TypeAnnotation::Number => "0.0".to_string(),
            TypeAnnotation::String => "rust::String()".to_string(),
            TypeAnnotation::Array(element_type) => {
                format!("rust::Vec<{}>()", element_type.as_cxx_type(mod_name)?)
            }
            TypeAnnotation::Enum(EnumTypeAnnotation { members, .. }) => {
                let enum_type = self.as_cxx_type(mod_name)?;
                let first_member = members
                    .first()
                    .ok_or(anyhow::anyhow!("Enum should have at least one member"))?;

                format!("{}::{}", enum_type, first_member.name)
            }
            TypeAnnotation::Object(..) => {
                let cxx_type = self.as_cxx_type(mod_name)?;
                format!("{}{{}}", cxx_type)
            }
            TypeAnnotation::Nullable(..) => {
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
            TypeAnnotation::Boolean
            | TypeAnnotation::Number
            | TypeAnnotation::String
            | TypeAnnotation::Array(..)
            | TypeAnnotation::Enum(..)
            | TypeAnnotation::Object(..)
            | TypeAnnotation::Nullable(..) => format!(
                "react::bridging::fromJs<{}>(rt, {}, callInvoker)",
                self.as_cxx_type(mod_name)?,
                ident
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
    pub fn as_cxx_to_js(&self, ident: &String) -> Result<CxxToJs, anyhow::Error> {
        let to_js_expr = match &*self {
            TypeAnnotation::Boolean
            | TypeAnnotation::Number
            | TypeAnnotation::String
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
    pub fn as_cxx_method(&self, mod_name: &String) -> Result<CxxMethod, anyhow::Error> {
        let vec_size = self.params.len() + 1;
        // ["arg0", "arg1", "arg2"]
        let mut args = Vec::with_capacity(vec_size);
        // ["auto arg0 = facebook::react::bridging::fromJs<T>(rt, value, callInvoker)", "..."]
        let mut args_decls = Vec::with_capacity(vec_size);

        args.push(RESERVED_ARG_NAME_ID.to_string());
        args_decls.push(format!(
            "uintptr_t {} = reinterpret_cast<uintptr_t>(&thisModule);",
            RESERVED_ARG_NAME_ID
        ));

        for (idx, param) in self.params.iter().enumerate() {
            let arg_ref = cxx_arg_ref(idx);
            let arg_var = cxx_arg_var(idx);
            let from_js = param.type_annotation.as_cxx_from_js(mod_name, &arg_ref)?;
            args.push(arg_var.clone());
            args_decls.push(format!("auto {} = {};", arg_var, from_js.expr));
        }

        let invoke_stmts = match &self.ret_type {
            TypeAnnotation::Promise(resolve_type) => {
                let fn_args = args.join(", ");
                let mut bind_args = Vec::with_capacity(args.len() + 2);
                bind_args.extend(args);
                bind_args.push("promise".to_string());

                let ret_stmts = if let TypeAnnotation::Void = &**resolve_type {
                    formatdoc! {
                        r#"
                        craby::bridging::{fn_name}({fn_args});
                        promise.resolve();
                        "#,
                        fn_name = self.name,
                        fn_args = fn_args,
                    }
                } else {
                    formatdoc! {
                        r#"
                        auto ret = craby::bridging::{fn_name}({fn_args});
                        promise.resolve(ret);
                        "#,
                        fn_name = self.name,
                        fn_args = fn_args,
                    }
                };

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
                    {ret_stmts}
                      }} catch (const jsi::JSError &err) {{
                        promise.reject(err.getMessage());
                      }} catch (const std::exception &err) {{
                        promise.reject(errorMessage(err));
                      }}
                    }}).detach();

                    return {ret};"#,
                    bind_args = bind_args.join(", "),
                    ret_stmts = indent_str(ret_stmts, 4),
                    ret_type = resolve_type.as_cxx_type(mod_name)?,
                    ret = self.ret_type.as_cxx_to_js(&"promise".to_string())?.expr,
                }
            }
            _ => {
                // Invoke the FFI function synchronously and return the result
                //
                // ```cpp
                // auto ret = craby::bridging::myFunc(arg0, arg1, arg2);
                // return ret;
                // ```
                let ret_stmts = if let TypeAnnotation::Void = &self.ret_type {
                    formatdoc! {
                        r#"craby::bridging::{fn_name}({fn_args});"#,
                        fn_name = self.name,
                        fn_args = args.join(", "),
                    }
                } else {
                    formatdoc! {
                        r#"auto ret = craby::bridging::{fn_name}({fn_args});"#,
                        fn_name = self.name,
                        fn_args = args.join(", "),
                    }
                };

                formatdoc! {
                    r#"
                    {ret_stmts}

                    return {to_js};"#,
                    ret_stmts = ret_stmts,
                    to_js = self.ret_type.as_cxx_to_js(&"ret".to_string())?.expr,
                }
            }
        };

        let args_decls = args_decls.join("\n");
        let cxx_mod = cxx_mod_cls_name(mod_name);
        let args_count = self.params.len();

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
    pub fn as_cxx_bridging_templates(&self) -> Result<Vec<String>, anyhow::Error> {
        let mut bridging_templates = BTreeMap::new();
        let mut enum_bridging_templates = BTreeMap::new();
        let mut nullable_bridging_templates = self.collect_nullable_types()?;

        self.aliases
            .iter()
            .try_for_each(|type_annotation| -> Result<(), anyhow::Error> {
                let alias_spec = type_annotation.as_object().unwrap();
                bridging_templates.insert(
                    alias_spec.name.clone(),
                    cxx_struct_bridging_template(&self.module_name, alias_spec)?,
                );
                Ok(())
            })?;

        self.enums
            .iter()
            .try_for_each(|type_annotation| -> Result<(), anyhow::Error> {
                let enum_spec = type_annotation.as_enum().unwrap();
                enum_bridging_templates.insert(
                    enum_spec.name.clone(),
                    cxx_enum_bridging_template(enum_spec)?,
                );
                Ok(())
            })?;

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
                nullable_bridging_templates.remove(&format!("craby::bridging::{}", name))
            {
                ordered_templates.push(template);
            }
        });

        ordered_templates.extend(bridging_templates.into_values());
        ordered_templates.extend(nullable_bridging_templates.into_values());

        Ok(ordered_templates)
    }

    pub fn collect_nullable_types(&self) -> Result<BTreeMap<String, String>, anyhow::Error> {
        // {
        //   "craby::bridging::NullableFoo": "(code)",
        //   "craby::bridging::NullableBar": "(code)",
        //   "craby::bridging::NullableBaz": "(code)",
        // }
        let mut nullable_bridging_templates = BTreeMap::new();

        self.methods
            .iter()
            .try_for_each(|method| -> Result<(), anyhow::Error> {
                method
                    .params
                    .iter()
                    .try_for_each(|param| -> Result<(), anyhow::Error> {
                        if let nullable_type @ TypeAnnotation::Nullable(type_annotation) =
                            &param.type_annotation
                        {
                            let key = nullable_type.as_cxx_type(&self.module_name)?;

                            if nullable_bridging_templates.contains_key(&key) {
                                return Ok(());
                            }

                            let bridging_template = cxx_nullable_bridging_template(
                                &self.module_name,
                                &nullable_type.as_cxx_type(&self.module_name)?,
                                type_annotation,
                            )?;

                            nullable_bridging_templates.insert(key, bridging_template);
                        }

                        Ok(())
                    })?;

                if let nullable_type @ TypeAnnotation::Nullable(type_annotation) = &method.ret_type
                {
                    let key = nullable_type.as_cxx_type(&self.module_name)?;

                    if nullable_bridging_templates.contains_key(&key) {
                        return Ok(());
                    }

                    let bridging_template = cxx_nullable_bridging_template(
                        &self.module_name,
                        &nullable_type.as_cxx_type(&self.module_name)?,
                        type_annotation,
                    )?;

                    nullable_bridging_templates.insert(key, bridging_template);
                }

                Ok(())
            })?;

        self.aliases
            .iter()
            .try_for_each(|type_annotation| -> Result<(), anyhow::Error> {
                let alias_spec = type_annotation.as_object().unwrap();
                alias_spec
                    .props
                    .iter()
                    .try_for_each(|prop| -> Result<(), anyhow::Error> {
                        match &prop.type_annotation {
                            nullable_type @ TypeAnnotation::Nullable(type_annotation) => {
                                let key = nullable_type.as_cxx_type(&self.module_name)?;

                                if nullable_bridging_templates.contains_key(&key) {
                                    return Ok(());
                                }

                                let bridging_template = cxx_nullable_bridging_template(
                                    &self.module_name,
                                    &nullable_type.as_cxx_type(&self.module_name)?,
                                    type_annotation,
                                )?;

                                nullable_bridging_templates.insert(key, bridging_template);

                                Ok(())
                            }
                            _ => Ok(()),
                        }
                    })?;
                Ok(())
            })?;

        Ok(nullable_bridging_templates)
    }
}

pub mod template {
    use indoc::formatdoc;

    use crate::{
        parser::types::{
            EnumMemberValue as ParserEnumMemberValue, EnumTypeAnnotation, ObjectTypeAnnotation,
            TypeAnnotation,
        },
        utils::indent_str,
    };

    pub fn cxx_struct_bridging_template(
        mod_name: &String,
        obj: &ObjectTypeAnnotation,
    ) -> Result<String, anyhow::Error> {
        let struct_namespace = format!("craby::bridging::{}", obj.name);
        let mut get_props = vec![];
        let mut set_props = vec![];
        let mut from_js_stmts = vec![];
        let mut from_js_ident = vec![];
        let mut to_js_stmts = vec![];

        obj.props
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

    pub fn cxx_enum_bridging_template(
        enum_spec: &EnumTypeAnnotation,
    ) -> Result<String, anyhow::Error> {
        let enum_namespace = format!("craby::bridging::{}", enum_spec.name);

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
                format!("\"{}\"", value)
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
              throw jsi::JSError(rt, "Invalid enum value ({name})");
            }}"#,
            name = enum_spec.name,
        });

        // ```cpp
        // default:
        //   throw jsi::JSError(rt, "Invalid enum value (MyEnum)");
        // ```
        to_js_conds.push(formatdoc! {
            r#"
            default:
              throw jsi::JSError(rt, "Invalid enum value ({name})");"#,
            name = enum_spec.name,
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

    pub fn cxx_arg_ref(idx: usize) -> String {
        format!("args[{}]", idx)
    }

    pub fn cxx_arg_var(idx: usize) -> String {
        format!("arg{}", idx)
    }
}
