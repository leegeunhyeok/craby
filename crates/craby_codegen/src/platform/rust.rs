use std::collections::BTreeMap;

use craby_common::utils::string::{pascal_case, snake_case};
use indoc::formatdoc;
use rustc_hash::FxHashMap;

use crate::{
    constants::specs::RESERVED_ARG_NAME_ID,
    parser::types::{
        EnumTypeAnnotation, Method, ObjectTypeAnnotation, Param, RefTypeAnnotation, TypeAnnotation,
    },
    platform::rust::template::{alias_default_impl, as_struct_def, enum_default_impl},
    types::Schema,
    utils::indent_str,
};

#[derive(Debug)]
pub struct RsType(pub String);

#[derive(Debug)]
pub struct RsBridgeType(pub String);

#[derive(Debug)]
pub struct RsImplType(pub String);

#[derive(Debug, Clone)]
pub struct RsCxxBridge {
    /// The struct definition.
    ///
    /// ```rust,ignore
    /// struct MyStruct {
    ///   foo: String,
    ///   bar: f64,
    ///   baz: bool,
    /// }
    /// ```
    pub struct_defs: Vec<String>,
    /// The enum definition.
    ///
    /// ```rust,ignore
    /// enum MyEnum {
    ///   Foo,
    ///   Bar,
    ///   Baz,
    /// }
    /// ```
    pub enum_defs: Vec<String>,
    /// The extern function declaration.
    ///
    /// **Example**
    ///
    /// ```rust,ignore
    /// #[cxx_name = "myFunc"]
    /// fn myFunc(arg1: Foo, arg2: Bar) -> Baz;
    /// ```
    pub func_extern_sigs: Vec<String>,
    /// The implementation function of the extern function.
    ///
    /// **Example**
    ///
    /// ```rust,ignore
    /// fn myFunc(arg1: Foo, arg2: Bar) -> Baz {
    ///   MyModule::my_func(arg1, arg2)
    /// }
    /// ```
    pub func_impls: Vec<String>,
}

impl TypeAnnotation {
    /// Returns the Rust type for the given `TypeAnnotation`.
    pub fn as_rs_type(&self) -> Result<RsType, anyhow::Error> {
        let rs_type = match self {
            TypeAnnotation::Void => "()".to_string(),
            TypeAnnotation::Boolean => "bool".to_string(),
            TypeAnnotation::Number => "f64".to_string(),
            TypeAnnotation::String => "String".to_string(),
            TypeAnnotation::Array(element_type) => {
                if let TypeAnnotation::Array(..) = &**element_type {
                    return Err(anyhow::anyhow!(
                        "Nested array type is not supported: {:?}",
                        element_type
                    ));
                }
                format!("Vec<{}>", element_type.as_rs_type()?.0)
            }
            TypeAnnotation::Object(ObjectTypeAnnotation { name, .. }) => name.clone(),
            TypeAnnotation::Enum(EnumTypeAnnotation { name, .. }) => name.clone(),
            TypeAnnotation::Promise(resolve_type) => {
                format!("Result<{}, anyhow::Error>", resolve_type.as_rs_type()?.0)
            }
            TypeAnnotation::Nullable(type_annotation) => match &**type_annotation {
                TypeAnnotation::Boolean => "NullableBoolean".to_string(),
                TypeAnnotation::Number => "NullableNumber".to_string(),
                TypeAnnotation::String => "NullableString".to_string(),
                TypeAnnotation::Object(ObjectTypeAnnotation { name, .. }) => {
                    format!("Nullable{}", name)
                }
                TypeAnnotation::Enum(EnumTypeAnnotation { name, .. }) => {
                    format!("Nullable{}", name)
                }
                TypeAnnotation::Ref(RefTypeAnnotation { name, .. }) => {
                    format!("Nullable{}", name)
                }
                TypeAnnotation::Array(element_type) => match &**element_type {
                    TypeAnnotation::Boolean => "NullableBooleanArray".to_string(),
                    TypeAnnotation::Number => "NullableNumberArray".to_string(),
                    TypeAnnotation::String => "NullableStringArray".to_string(),
                    TypeAnnotation::Object(ObjectTypeAnnotation { name, .. }) => {
                        format!("Nullable{}Array", name)
                    }
                    TypeAnnotation::Enum(EnumTypeAnnotation { name, .. }) => {
                        format!("Nullable{}Array", name)
                    }
                    TypeAnnotation::Ref(RefTypeAnnotation { name, .. }) => {
                        format!("Nullable{}Array", name)
                    }
                    _ => {
                        return Err(anyhow::anyhow!(
                        "[as_rs_type] Unsupported type annotation for nullable array type: {:?}",
                        element_type
                    ))
                    }
                },
                _ => {
                    return Err(anyhow::anyhow!(
                        "[as_rs_type] Unsupported type annotation for nullable type: {:?}",
                        type_annotation
                    ))
                }
            },
            _ => {
                return Err(anyhow::anyhow!(
                    "[as_rs_type] Unsupported type annotation: {:?}",
                    self
                ));
            }
        };

        Ok(RsType(rs_type))
    }

    /// Returns the Rust type for the given `TypeAnnotation` that is used in the cxx extern function.
    pub fn as_rs_bridge_type(&self) -> Result<RsBridgeType, anyhow::Error> {
        let extern_type = match self {
            TypeAnnotation::Promise(resolve_type) => {
                format!("Result<{}>", resolve_type.as_rs_type()?.0)
            }
            _ => self.as_rs_type()?.0,
        };

        Ok(RsBridgeType(extern_type))
    }

    pub fn as_rs_impl_type(&self) -> Result<RsImplType, anyhow::Error> {
        let rs_type = match self {
            TypeAnnotation::Void => "Void".to_string(),
            TypeAnnotation::Boolean => "Boolean".to_string(),
            TypeAnnotation::Number => "Number".to_string(),
            TypeAnnotation::String => "String".to_string(),
            TypeAnnotation::Array(element_type) => {
                if let TypeAnnotation::Array { .. } = &**element_type {
                    return Err(anyhow::anyhow!(
                        "Nested array type is not supported: {:?}",
                        element_type
                    ));
                }
                format!("Array<{}>", element_type.as_rs_impl_type()?.0)
            }
            TypeAnnotation::Object(ObjectTypeAnnotation { name, .. }) => name.clone(),
            TypeAnnotation::Enum(EnumTypeAnnotation { name, .. }) => name.clone(),
            TypeAnnotation::Promise(resolved_type) => {
                format!("Promise<{}>", resolved_type.as_rs_impl_type()?.0)
            }
            TypeAnnotation::Nullable(type_annotation) => {
                let type_annotation = type_annotation.as_rs_impl_type()?.0;
                format!("Nullable<{}>", type_annotation)
            }
            TypeAnnotation::Ref(..) => unreachable!(),
        };
        Ok(RsImplType(rs_type))
    }

    pub fn as_rs_default_val(&self) -> Result<String, anyhow::Error> {
        let default_val = match self {
            TypeAnnotation::Boolean => "false".to_string(),
            TypeAnnotation::Number => "0.0".to_string(),
            TypeAnnotation::String => "String::default()".to_string(),
            TypeAnnotation::Array(..) => "Vec::default()".to_string(),
            TypeAnnotation::Enum(EnumTypeAnnotation { name, .. }) => {
                format!("{}::default()", name)
            }
            TypeAnnotation::Object(ObjectTypeAnnotation { name, .. }) => {
                format!("{}::default()", name)
            }
            TypeAnnotation::Nullable(..) => {
                let nullable_type = self.as_rs_type()?.0;
                format!("{}::default()", nullable_type)
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "[as_rs_default_val] Unsupported type annotation: {:?}",
                    self
                ))
            }
        };

        Ok(default_val)
    }
}

impl Method {
    pub fn as_impl_sig(&self) -> Result<String, anyhow::Error> {
        let return_type = self.ret_type.as_rs_impl_type()?.0;
        let params_sig = self
            .params
            .iter()
            .map(|param| param.as_impl_sig())
            .collect::<Result<Vec<_>, _>>()
            .map(|params| params.join(", "))?;

        let fn_name = snake_case(&self.name);
        let ret_annotation = if return_type == "()" {
            String::new()
        } else {
            format!(" -> {}", return_type)
        };

        Ok(format!(
            "fn {}(&self, {}){}",
            fn_name.to_string(),
            params_sig,
            ret_annotation
        ))
    }
}

impl Param {
    pub fn as_cxx_sig(&self) -> Result<String, anyhow::Error> {
        let param_type = self.type_annotation.as_rs_type()?.0;
        Ok(format!("{}: {}", self.name, param_type))
    }

    pub fn as_impl_sig(&self) -> Result<String, anyhow::Error> {
        let param_type = self.type_annotation.as_rs_impl_type()?.0;
        Ok(format!("{}: {}", self.name, param_type))
    }
}

impl Schema {
    /// Returns the Rust cxx bridging function declaration and implementation for the `FunctionSpec`.
    pub fn as_rs_cxx_bridge(&self) -> Result<RsCxxBridge, anyhow::Error> {
        let mut func_extern_sigs = vec![];
        let mut func_impls = vec![];
        let mut type_impls = vec![];
        let mut struct_defs = FxHashMap::default();

        // Collect extern function signatures and implementations
        self.methods
            .iter()
            .try_for_each(|method_spec| -> Result<(), anyhow::Error> {
                // Collect nullable parameters
                method_spec
                    .params
                    .iter()
                    .try_for_each(|param| -> Result<(), anyhow::Error> {
                        if let nullable_type @ TypeAnnotation::Nullable(type_annotation) =
                            &param.type_annotation
                        {
                            if struct_defs.contains_key(nullable_type) {
                                return Ok(());
                            }

                            let struct_type = nullable_type.as_rs_bridge_type()?.0;
                            let base_type = type_annotation.as_rs_type()?.0;
                            let rs_impl_type = type_annotation.as_rs_impl_type()?.0;
                            let default_val = type_annotation.as_rs_default_val()?;

                            struct_defs.insert(nullable_type.clone(), formatdoc! {
                                r#"
                                struct {struct_type} {{
                                    null: bool,
                                    val: {base_type},
                                }}"#,
                                struct_type = struct_type,
                                base_type = base_type,
                            });

                            let nullable_impl = formatdoc! {
                                r#"
                                impl From<{struct_type}> for Nullable<{rs_impl_type}> {{
                                    fn from(val: {struct_type}) -> Self {{
                                        Nullable::new(if val.null {{ None }} else {{ Some(val.val) }})
                                    }}
                                }}

                                impl From<Nullable<{rs_impl_type}>> for {struct_type} {{
                                    fn from(val: Nullable<{rs_impl_type}>) -> Self {{
                                        let val = val.into_value();
                                        let null = val.is_none();
                                        {struct_type} {{
                                            val: val.unwrap_or({default_val}),
                                            null,
                                        }}
                                    }}
                                }}"#,
                                struct_type = struct_type,
                                rs_impl_type = rs_impl_type,
                                default_val = default_val,
                            };

                            type_impls.push(nullable_impl);
                        }

                        Ok(())
                    })?;

                let ret_type = method_spec.ret_type.as_rs_type()?.0;
                let ret_extern_type = method_spec.ret_type.as_rs_bridge_type()?.0;

                let params_sig = method_spec
                    .params
                    .iter()
                    .map(|param| param.as_cxx_sig())
                    .collect::<Result<Vec<_>, _>>()
                    .map(|mut params| {
                        params.insert(0, format!("{}: usize", RESERVED_ARG_NAME_ID));
                        params.join(", ")
                    })?;

                let impl_name = pascal_case(&self.module_name);
                let mod_name = snake_case(&self.module_name);
                let fn_name = snake_case(&method_spec.name);
                let fn_args = method_spec
                    .params
                    .iter()
                    .map(|param| {
                        if let TypeAnnotation::Nullable(..) = &param.type_annotation {
                            format!("{}.into()", param.name)
                        } else {
                            param.name.clone()
                        }
                    })
                    .collect::<Vec<_>>();
                let prefixed_fn_name = format!("{}_{}", mod_name, fn_name);

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
                    fn {prefixed_fn_name}({params_sig}){ret};"#,
                    orig_fn_name = method_spec.name,
                    prefixed_fn_name = prefixed_fn_name,
                    params_sig = params_sig,
                    ret = ret_extern_annotation,
                };

                let ret = if let TypeAnnotation::Nullable(..) = &method_spec.ret_type {
                    "ret.into()"
                } else {
                    "ret"
                };

                let impl_func = formatdoc! {
                    r#"
                    fn {prefixed_fn_name}({params_sig}){ret_type} {{
                        let it = {impl_name}::new({id});
                        let ret = it.{fn_name}({fn_args});
                        {ret}
                    }}"#,
                    params_sig = params_sig,
                    ret_type = ret_annotation,
                    impl_name = impl_name,
                    id = RESERVED_ARG_NAME_ID,
                    prefixed_fn_name = prefixed_fn_name,
                    fn_name = fn_name.to_string(),
                    fn_args = fn_args.join(", "),
                };

                func_extern_sigs.push(extern_func);
                func_impls.push(impl_func);

                Ok(())
            })?;

        // Collect alias types (struct)
        self.aliases
            .iter()
            .try_for_each(|type_annotation| -> Result<(), anyhow::Error> {
                if struct_defs.contains_key(type_annotation) {
                    return Ok(());
                }

                let obj = type_annotation.as_object().unwrap();
                struct_defs.insert(type_annotation.clone(), as_struct_def(obj)?);
                type_impls.push(alias_default_impl(obj)?);
                Ok(())
            })?;

        // Collect enum types
        let enum_defs = self
            .enums
            .iter()
            .map(|type_annotation| {
                let enum_schema = type_annotation.as_enum().unwrap();
                let members = enum_schema
                    .members
                    .iter()
                    .map(|m| format!("{},", m.name))
                    .collect::<Vec<_>>()
                    .join("\n");

                formatdoc! {
                    r#"
                    enum {name} {{
                    {members}
                    }}"#,
                    name = enum_schema.name,
                    members = indent_str(members, 4),
                }
            })
            .collect();

        Ok(RsCxxBridge {
            struct_defs: struct_defs.into_values().collect(),
            enum_defs,
            func_extern_sigs,
            func_impls,
        })
    }

    pub fn as_rs_type_impls(
        &self,
        type_impls: &mut BTreeMap<String, String>,
    ) -> Result<(), anyhow::Error> {
        // Collect extern function signatures and implementations
        self
            .methods
            .iter()
            .try_for_each(|method_spec| -> Result<(), anyhow::Error> {
                method_spec.params.iter().try_for_each(|param| -> Result<(), anyhow::Error> {
                    // Collect nullable parameters
                    if let nullable_type @ TypeAnnotation::Nullable(type_annotation) = &param.type_annotation
                    {
                        let rs_type = type_annotation.as_rs_type()?.0;

                        if !type_impls.contains_key(&rs_type) {
                            let nullable_type = nullable_type.as_rs_bridge_type()?.0;
                            let rs_impl_type = type_annotation.as_rs_impl_type()?.0;
                            let default_val = type_annotation.as_rs_default_val()?;

                            let default_impl = formatdoc! {
                                r#"
                                impl Default for {nullable_type} {{
                                    fn default() -> Self {{
                                        {nullable_type} {{
                                            null: true,
                                            val: {default_val},
                                        }}
                                    }}
                                }}"#,
                                nullable_type = nullable_type,
                                default_val = default_val,
                            };

                            let nullable_impl = formatdoc! {
                                r#"
                                impl From<{nullable_type}> for Nullable<{rs_impl_type}> {{
                                    fn from(val: {nullable_type}) -> Self {{
                                        Nullable::new(if val.null {{ None }} else {{ Some(val.val) }})
                                    }}
                                }}

                                impl From<Nullable<{rs_impl_type}>> for {nullable_type} {{
                                    fn from(val: Nullable<{rs_impl_type}>) -> Self {{
                                        let val = val.into_value();
                                        let null = val.is_none();
                                        {nullable_type} {{
                                            val: val.unwrap_or({default_val}),
                                            null,
                                        }}
                                    }}
                                }}"#,
                                rs_impl_type = rs_impl_type,
                                nullable_type = nullable_type,
                                default_val = default_val,
                            };

                            type_impls.insert(
                                rs_type,
                                [default_impl, nullable_impl].join("\n\n"),
                            );
                        }
                    }

                    Ok(())
                })?;

                if let nullable_type @ TypeAnnotation::Nullable(type_annotation) = &method_spec.ret_type {
                    let rs_type = type_annotation.as_rs_type()?.0;

                    if !type_impls.contains_key(&rs_type) {
                        let nullable_type = nullable_type.as_rs_bridge_type()?.0;
                        let rs_impl_type = type_annotation.as_rs_impl_type()?.0;
                        let default_val = type_annotation.as_rs_default_val()?;

                        let default_impl = formatdoc! {
                            r#"
                            impl Default for {nullable_type} {{
                                fn default() -> Self {{
                                    {nullable_type} {{
                                        null: true,
                                        val: {default_val},
                                    }}
                                }}
                            }}"#,
                            nullable_type = nullable_type,
                            default_val = default_val,
                        };

                        let nullable_impl = formatdoc! {
                            r#"
                            impl From<{nullable_type}> for Nullable<{rs_impl_type}> {{
                                fn from(val: {nullable_type}) -> Self {{
                                    Nullable::new(if val.null {{ None }} else {{ Some(val.val) }})
                                }}
                            }}

                            impl From<Nullable<{rs_impl_type}>> for {nullable_type} {{
                                fn from(val: Nullable<{rs_impl_type}>) -> Self {{
                                    let val = val.into_value();
                                    let null = val.is_none();
                                    {nullable_type} {{
                                        val: val.unwrap_or({default_val}),
                                        null,
                                    }}
                                }}
                            }}"#,
                            rs_impl_type = rs_impl_type,
                            nullable_type = nullable_type,
                            default_val = default_val,
                        };

                        type_impls
                            .insert(rs_type, [default_impl, nullable_impl].join("\n\n"));
                    }
                }

                Ok(())
            })?;

        // impl Default trait for the alias type
        self.aliases
            .iter()
            .try_for_each(|type_annotation| -> Result<(), anyhow::Error> {
                let obj = type_annotation.as_object().unwrap();
                if type_impls.contains_key(&obj.name) {
                    return Ok(());
                }
                type_impls.insert(obj.name.clone(), alias_default_impl(obj)?);
                Ok(())
            })?;

        // Collect enum types
        self.enums
            .iter()
            .try_for_each(|type_annotation| -> Result<(), anyhow::Error> {
                let enum_schema = type_annotation.as_enum().unwrap();
                if type_impls.contains_key(&enum_schema.name) {
                    return Ok(());
                }
                type_impls.insert(enum_schema.name.clone(), enum_default_impl(enum_schema)?);
                Ok(())
            })?;

        Ok(())
    }
}

pub mod template {
    use indoc::formatdoc;

    use crate::{
        parser::types::{EnumTypeAnnotation, ObjectTypeAnnotation, TypeAnnotation},
        utils::indent_str,
    };

    pub fn as_struct_def(obj: &ObjectTypeAnnotation) -> Result<String, anyhow::Error> {
        let mut struct_defs = vec![];

        // Example:
        // ```
        // foo: String,
        // bar: f64,
        // baz: bool,
        // ```
        let props = obj
            .props
            .iter()
            .map(|prop| -> Result<String, anyhow::Error> {
                Ok(format!(
                    "{}: {},",
                    prop.name,
                    prop.type_annotation.as_rs_bridge_type()?.0
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;

        obj.props
            .iter()
            .try_for_each(|prop| -> Result<(), anyhow::Error> {
                if let nullable_type @ TypeAnnotation::Nullable(type_annotation) =
                    &prop.type_annotation
                {
                    let name = nullable_type.as_rs_bridge_type()?.0;
                    let rs_type = type_annotation.as_rs_bridge_type()?.0;
                    let struct_def = formatdoc! {
                        r#"
                        struct {name} {{
                            null: bool,
                            val: {rs_type}
                        }}"#,
                        name = name,
                        rs_type = rs_type,
                    };

                    struct_defs.push(struct_def);
                }

                Ok(())
            })?;

        let struct_def = formatdoc! {
            r#"
            struct {name} {{
            {props}
            }}"#,
            name = obj.name,
            props = indent_str(props.join("\n"), 4),
        };

        struct_defs.push(struct_def);

        Ok(struct_defs.join("\n\n"))
    }

    pub fn alias_default_impl(obj: &ObjectTypeAnnotation) -> Result<String, anyhow::Error> {
        let mut default_impls = vec![];

        let props_with_default_val = obj
            .props
            .iter()
            .map(|prop| -> Result<String, anyhow::Error> {
                Ok(format!(
                    "{}: {}",
                    prop.name,
                    prop.type_annotation.as_rs_default_val()?
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let default_impl = formatdoc! {
            r#"
            impl Default for {name} {{
                fn default() -> Self {{
                    {name} {{
            {props}
                    }}
                }}
            }}"#,
            name = obj.name,
            props = indent_str(props_with_default_val.join(",\n"), 12),
        };

        obj.props
            .iter()
            .try_for_each(|property| -> Result<(), anyhow::Error> {
                if let nullable_type @ TypeAnnotation::Nullable(type_annotation) =
                    &property.type_annotation
                {
                    let nullable_type = nullable_type.as_rs_bridge_type()?.0;
                    let rs_impl_type = type_annotation.as_rs_impl_type()?.0;
                    let default_val = type_annotation.as_rs_default_val()?;

                    let default_impl = formatdoc! {
                        r#"
                        impl Default for {nullable_type} {{
                            fn default() -> Self {{
                                {nullable_type} {{
                                    null: true,
                                    val: {default_val},
                                }}
                            }}
                        }}"#,
                        nullable_type = nullable_type,
                        default_val = default_val,
                    };

                    let nullable_impl = formatdoc! {
                        r#"
                        impl From<{nullable_type}> for Nullable<{rs_impl_type}> {{
                            fn from(val: {nullable_type}) -> Self {{
                                Nullable::new(if val.null {{ None }} else {{ Some(val.val) }})
                            }}
                        }}

                        impl From<Nullable<{rs_impl_type}>> for {nullable_type} {{
                            fn from(val: Nullable<{rs_impl_type}>) -> Self {{
                                let val = val.into_value();
                                let null = val.is_none();
                                {nullable_type} {{
                                    val: val.unwrap_or({default_val}),
                                    null,
                                }}
                            }}
                        }}"#,
                        rs_impl_type = rs_impl_type,
                        nullable_type = nullable_type,
                        default_val = default_val,
                    };

                    default_impls.push(default_impl);
                    default_impls.push(nullable_impl);
                }

                Ok(())
            })?;

        default_impls.push(default_impl);

        Ok(default_impls.join("\n\n"))
    }

    pub fn enum_default_impl(enum_schema: &EnumTypeAnnotation) -> Result<String, anyhow::Error> {
        let first_member = enum_schema
            .members
            .first()
            .expect("Enum members are required")
            .name
            .clone();

        let name = enum_schema.name.clone();

        Ok(formatdoc! {
            r#"
            impl Default for {name} {{
                fn default() -> Self {{
                    {name}::{first_member}
                }}
            }}"#,
            name = name,
            first_member = first_member
        })
    }
}
