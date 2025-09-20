use std::collections::BTreeMap;

use craby_common::utils::string::{flat_case, pascal_case, snake_case};
use indoc::formatdoc;
use template::{alias_default_impl, alias_struct_def, enum_default_impl};

use crate::{
    types::{
        schema::{Schema, TypeAnnotation},
        types::CodegenResult,
    },
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
            // Boolean type
            TypeAnnotation::BooleanTypeAnnotation => "bool".to_string(),

            // Number types
            TypeAnnotation::NumberTypeAnnotation
            | TypeAnnotation::FloatTypeAnnotation
            | TypeAnnotation::DoubleTypeAnnotation
            | TypeAnnotation::Int32TypeAnnotation
            | TypeAnnotation::NumberLiteralTypeAnnotation { .. } => "f64".to_string(),

            // String types
            TypeAnnotation::StringTypeAnnotation
            | TypeAnnotation::StringLiteralTypeAnnotation { .. }
            | TypeAnnotation::StringLiteralUnionTypeAnnotation { .. } => "String".to_string(),

            // Array type
            TypeAnnotation::ArrayTypeAnnotation { element_type } => {
                if let TypeAnnotation::ArrayTypeAnnotation { .. } = &**element_type {
                    return Err(anyhow::anyhow!(
                        "Nested array type is not supported: {:?}",
                        element_type
                    ));
                }
                format!("Vec<{}>", element_type.as_rs_type()?.0)
            }

            // Type alias
            TypeAnnotation::TypeAliasTypeAnnotation { name } => name.clone(),

            // Enum
            TypeAnnotation::EnumDeclaration { name, .. } => name.clone(),

            // Promise type
            TypeAnnotation::PromiseTypeAnnotation { element_type } => {
                format!("Result<{}, anyhow::Error>", element_type.as_rs_type()?.0)
            }

            // Nullable type
            TypeAnnotation::NullableTypeAnnotation { type_annotation } => {
                match &**type_annotation {
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
                }
            }

            // Void type
            TypeAnnotation::VoidTypeAnnotation => "()".to_string(),

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
            TypeAnnotation::PromiseTypeAnnotation { element_type } => {
                format!("Result<{}>", element_type.as_rs_type()?.0)
            }
            _ => self.as_rs_type()?.0,
        };

        Ok(RsBridgeType(extern_type))
    }

    pub fn as_rs_impl_type(&self) -> Result<RsImplType, anyhow::Error> {
        let rs_type = match self {
            // Boolean type
            TypeAnnotation::BooleanTypeAnnotation => "Boolean".to_string(),

            // Number types
            TypeAnnotation::NumberTypeAnnotation
            | TypeAnnotation::FloatTypeAnnotation
            | TypeAnnotation::DoubleTypeAnnotation
            | TypeAnnotation::Int32TypeAnnotation
            | TypeAnnotation::NumberLiteralTypeAnnotation { .. } => "Number".to_string(),

            // String types
            TypeAnnotation::StringTypeAnnotation
            | TypeAnnotation::StringLiteralTypeAnnotation { .. }
            | TypeAnnotation::StringLiteralUnionTypeAnnotation { .. } => "String".to_string(),

            // Array type
            TypeAnnotation::ArrayTypeAnnotation { element_type } => {
                if let TypeAnnotation::ArrayTypeAnnotation { .. } = &**element_type {
                    return Err(anyhow::anyhow!(
                        "Nested array type is not supported: {:?}",
                        element_type
                    ));
                }
                format!("Array<{}>", element_type.as_rs_impl_type()?.0)
            }

            // Type alias
            TypeAnnotation::TypeAliasTypeAnnotation { name } => name.clone(),

            // Enum
            TypeAnnotation::EnumDeclaration { name, .. } => name.clone(),

            // Promise type
            TypeAnnotation::PromiseTypeAnnotation { element_type } => {
                format!("Promise<{}>", element_type.as_rs_impl_type()?.0)
            }

            // Nullable type
            TypeAnnotation::NullableTypeAnnotation { type_annotation } => {
                let type_annotation = type_annotation.as_rs_impl_type()?.0;
                format!("Nullable<{}>", type_annotation)
            }

            // Void type
            TypeAnnotation::VoidTypeAnnotation => "Void".to_string(),

            _ => {
                return Err(anyhow::anyhow!(
                    "[as_rs_impl_type] Unsupported type annotation: {:?}",
                    self
                ));
            }
        };
        Ok(RsImplType(rs_type))
    }

    pub fn as_rs_default_val(&self) -> Result<String, anyhow::Error> {
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
                "String::default()".to_string()
            }

            // Array type
            TypeAnnotation::ArrayTypeAnnotation { .. } => "Vec::default()".to_string(),

            // Enum
            TypeAnnotation::EnumDeclaration { name, .. } => format!("{}::default()", name),

            // Type alias
            TypeAnnotation::TypeAliasTypeAnnotation { name } => format!("{}::default()", name),

            // Nullable type
            TypeAnnotation::NullableTypeAnnotation { .. } => {
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

impl Schema {
    /// Returns the Rust cxx bridging function declaration and implementation for the `FunctionSpec`.
    pub fn as_rs_cxx_bridge(&self) -> Result<RsCxxBridge, anyhow::Error> {
        let mut func_extern_sigs = vec![];
        let mut func_impls = vec![];
        let mut struct_defs = vec![];
        let mut enum_defs = vec![];
        let mut type_impls = vec![];

        // Collect extern function signatures and implementations
        self.spec
            .methods
            .iter()
            .try_for_each(|spec| -> Result<(), anyhow::Error> {
                match &*spec.type_annotation {
                    TypeAnnotation::FunctionTypeAnnotation {
                        return_type_annotation,
                        params,
                    } => {
                        // Validate optional parameters and return type
                        if spec.optional {
                            return Err(anyhow::anyhow!(
                                "Optional method is not supported: {}",
                                spec.name
                            ));
                        }

                        params.iter().try_for_each(|param| {
                            if param.optional {
                                return Err(anyhow::anyhow!(
                                    "Optional parameter is not supported: {}",
                                    param.name
                                ));
                            }

                            // Collect nullable parameters
                            if let nullable_type @ TypeAnnotation::NullableTypeAnnotation {
                                type_annotation,
                            } = &*param.type_annotation
                            {
                                let nullable_type = nullable_type.as_rs_bridge_type()?.0;
                                let rs_type = type_annotation.as_rs_type()?.0;
                                let rs_impl_type = type_annotation.as_rs_impl_type()?.0;
                                let default_val = type_annotation.as_rs_default_val()?;

                                struct_defs.push(formatdoc! {
                                    r#"
                                    struct {nullable_type} {{
                                        null: bool,
                                        val: {rs_type},
                                    }}"#,
                                    nullable_type = nullable_type,
                                    rs_type = rs_type,
                                });

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

                                type_impls.push(nullable_impl);
                            }

                            Ok(())
                        })?;

                        let ret_type = return_type_annotation.as_rs_type()?.0;
                        let ret_extern_type = return_type_annotation.as_rs_bridge_type()?.0;

                        let params_sig = params
                            .iter()
                            .map(|param| param.as_cxx_sig())
                            .collect::<Result<Vec<_>, _>>()
                            .map(|params| params.join(", "))?;

                        let impl_name = pascal_case(&self.module_name);
                        let mod_name = snake_case(&self.module_name);
                        let fn_name = snake_case(&spec.name);
                        let fn_args = params
                            .iter()
                            .map(|p| {
                                if let TypeAnnotation::NullableTypeAnnotation { .. } =
                                    &*p.type_annotation
                                {
                                    format!("{}.into()", p.name)
                                } else {
                                    p.name.clone()
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
                            orig_fn_name = spec.name,
                            prefixed_fn_name = prefixed_fn_name,
                            params_sig = params_sig,
                            ret = ret_extern_annotation,
                        };

                        let ret = if let TypeAnnotation::NullableTypeAnnotation { .. } =
                            &**return_type_annotation
                        {
                            "ret.into()"
                        } else {
                            "ret"
                        };

                        let impl_func = formatdoc! {
                            r#"
                            fn {prefixed_fn_name}({params_sig}){ret_type} {{
                                let ret = {impl_name}::{fn_name}({fn_args});
                                {ret}
                            }}"#,
                            params_sig = params_sig,
                            ret_type = ret_annotation,
                            impl_name = impl_name,
                            prefixed_fn_name = prefixed_fn_name,
                            fn_name = fn_name.to_string(),
                            fn_args = fn_args.join(", "),
                        };

                        func_extern_sigs.push(extern_func);
                        func_impls.push(impl_func);

                        Ok(())
                    }
                    _ => {
                        return Err(anyhow::anyhow!(
                            "[as_rs_cxx_bridge] Unsupported type annotation for function: {}",
                            spec.name
                        ))
                    }
                }
            })?;

        // Collect alias types (struct)
        self.alias_map.iter().try_for_each(
            |(name, alias_schema)| -> Result<(), anyhow::Error> {
                struct_defs.push(alias_struct_def(name, alias_schema)?);
                type_impls.push(alias_default_impl(name, alias_schema)?);
                Ok(())
            },
        )?;

        // Collect enum types
        self.enum_map
            .iter()
            .try_for_each(|(_, enum_schema)| -> Result<(), anyhow::Error> {
                let mut member_defs = vec![];

                match &enum_schema.members {
                    Some(members) => {
                        members
                            .iter()
                            .try_for_each(|member| -> Result<(), anyhow::Error> {
                                let member_def = format!("{},", member.name);
                                member_defs.push(member_def);
                                Ok(())
                            })?;
                    }
                    None => {
                        return Err(anyhow::anyhow!("Enum members are required"));
                    }
                }

                let enum_def = formatdoc! {
                    r#"
                    enum {name} {{
                    {members}
                    }}"#,
                    name = enum_schema.name,
                    members = indent_str(member_defs.join("\n"), 4),
                };

                enum_defs.push(enum_def);

                Ok(())
            })?;

        Ok(RsCxxBridge {
            struct_defs,
            enum_defs,
            func_extern_sigs,
            func_impls,
        })
    }

    pub fn as_rs_type_impls(&self) -> Result<BTreeMap<String, String>, anyhow::Error> {
        let mut type_impls = BTreeMap::new();

        // Collect extern function signatures and implementations
        self.spec
            .methods
            .iter()
            .try_for_each(|spec| -> Result<(), anyhow::Error> {
                match &*spec.type_annotation {
                    TypeAnnotation::FunctionTypeAnnotation {
                        return_type_annotation,
                        params,
                    } => {
                        params.iter().try_for_each(|param| {
                            if param.optional {
                                return Err(anyhow::anyhow!(
                                    "Optional parameter is not supported: {}",
                                    param.name
                                ));
                            }

                            // Collect nullable parameters
                            if let nullable_type @ TypeAnnotation::NullableTypeAnnotation {
                                type_annotation,
                            } = &*param.type_annotation
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
                                                    val: {default_val}
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

                        if let TypeAnnotation::NullableTypeAnnotation { type_annotation } =
                            &**return_type_annotation
                        {
                            let rs_type = type_annotation.as_rs_type()?.0;

                            if !type_impls.contains_key(&rs_type) {
                                let nullable_type = type_annotation.as_rs_bridge_type()?.0;
                                let rs_impl_type = type_annotation.as_rs_impl_type()?.0;
                                let default_val = type_annotation.as_rs_default_val()?;

                                let default_impl = formatdoc! {
                                    r#"
                                    impl Default for {nullable_type} {{
                                        fn default() -> Self {{
                                            {nullable_type} {{
                                                null: true,
                                                val: {default_val}
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
                    }
                    _ => {
                        return Err(anyhow::anyhow!(
                            "[as_rs_type_impls] Unsupported type annotation: {}",
                            spec.name
                        ))
                    }
                }
            })?;

        // impl Default trait for the alias type
        self.alias_map.iter().try_for_each(
            |(name, alias_schema)| -> Result<(), anyhow::Error> {
                if !type_impls.contains_key(name) {
                    type_impls.insert(name.clone(), alias_default_impl(name, alias_schema)?);
                }
                Ok(())
            },
        )?;

        // Collect enum types
        self.enum_map
            .iter()
            .try_for_each(|(name, enum_schema)| -> Result<(), anyhow::Error> {
                if !type_impls.contains_key(name) {
                    type_impls.insert(name.clone(), enum_default_impl(name, enum_schema)?);
                }
                Ok(())
            })?;

        Ok(type_impls)
    }
}

fn cxx_bridging_extern(codegen_res: &Vec<CodegenResult>) -> Vec<String> {
    codegen_res
        .iter()
        .map(|res| {
            let flat_name = flat_case(&res.module_name);
            let cxx_extern = res.rs_cxx_bridge.func_extern_sigs.join("\n\n");
            let struct_defs = res.rs_cxx_bridge.struct_defs.join("\n\n");
            let enum_defs = res.rs_cxx_bridge.enum_defs.join("\n\n");

            formatdoc! {
                r#"
                #[cxx::bridge(namespace = "craby::{flat_name}")]
                pub mod bridging {{
                    // Type definitions
                {struct_defs}

                {enum_defs}

                    extern "Rust" {{
                {cxx_extern}
                    }}
                }}"#,
                flat_name = flat_name,
                struct_defs = indent_str(struct_defs, 4),
                enum_defs = indent_str(enum_defs, 4),
                cxx_extern = indent_str(cxx_extern, 8),
            }
        })
        .collect::<Vec<_>>()
}

pub mod template {
    use craby_common::constants::impl_mod_name;
    use indoc::formatdoc;

    use crate::{
        platform::rust::cxx_bridging_extern,
        types::{
            schema::{Alias, Enum, TypeAnnotation},
            types::CodegenResult,
        },
        utils::indent_str,
    };

    /// Generate the `lib.rs` file for the given code generation results.
    ///
    /// ```rust,ignore
    /// pub(crate) mod generated;
    /// pub(crate) mod ffi;
    /// pub(crate) mod my_module_impl;
    /// ```
    pub fn lib_rs(codgen_res: &Vec<CodegenResult>) -> String {
        let impl_mods = codgen_res
            .iter()
            .map(|res| format!("pub(crate) mod {};", res.impl_mod))
            .collect::<Vec<String>>();

        formatdoc! {
            r#"
            #[rustfmt::skip]
            pub(crate) mod ffi;
            pub(crate) mod generated;
            pub(crate) mod types;

            {impl_mods}"#,
            impl_mods = impl_mods.join("\n"),
        }
    }

    /// Generate the `ffi.rs` file for the given code generation results.
    ///
    /// ```rust,ignore
    /// use ffi::*;
    /// use crate::generated::*;
    /// use crate::my_module_impl::*;
    ///
    /// #[cxx::bridge(namespace = "craby::mymodule")]
    /// pub mod bridging {
    ///     extern "Rust" {
    ///         #[cxx_name = "numericMethod"]
    ///         fn my_module_numeric_method(arg: f64) -> f64;
    ///     }
    /// }
    ///
    /// fn my_module_numeric_method(arg: f64) -> f64 {
    ///     MyModule::numeric_method(arg)
    /// }
    /// ```
    pub fn ffi_rs(codgen_res: &Vec<CodegenResult>) -> String {
        let impl_mods = codgen_res
            .iter()
            .map(|res| format!("use crate::{}::*;", impl_mod_name(&res.module_name)))
            .collect::<Vec<_>>();

        let cxx_externs = cxx_bridging_extern(&codgen_res);
        let cxx_impls = cxx_bridging_impl(&codgen_res);

        formatdoc! {
            r#"
            #[rustfmt::skip]
            {impl_mods}
            use crate::generated::*;

            use bridging::*;

            {cxx_extern}

            {cxx_impl}"#,
            impl_mods = impl_mods.join("\n"),
            cxx_extern = cxx_externs.join("\n\n"),
            cxx_impl = cxx_impls.join("\n\n"),
        }
    }

    pub fn types_rs() -> String {
        formatdoc! {
            r#"
            #[rustfmt::skip]
            pub type Boolean = bool;
            pub type Number = f64;
            pub type String = std::string::String;
            pub type Array<T> = Vec<T>;
            pub type Promise<T> = Result<T, anyhow::Error>;
            pub type Void = ();

            pub mod promise {{
                use super::Promise;

                pub fn resolve<T>(val: T) -> Promise<T> {{
                    Ok(val)
                }}

                pub fn rejected<T>(err: impl AsRef<str>) -> Promise<T> {{
                    Err(anyhow::anyhow!(err.as_ref().to_string()))
                }}
            }}

            pub struct Nullable<T> {{
                val: Option<T>,
            }}

            impl<T> Nullable<T> {{
                pub fn new(val: Option<T>) -> Self {{
                    Nullable {{ val }}
                }}

                pub fn some(val: T) -> Self {{
                    Nullable {{ val: Some(val) }}
                }}

                pub fn none() -> Self {{
                    Nullable {{ val: None }}
                }}

                pub fn value(mut self, val: T) -> Self {{
                    self.val = Some(val);
                    self
                }}

                pub fn value_of(&self) -> Option<&T> {{
                    self.val.as_ref()
                }}

                pub fn into_value(self) -> Option<T> {{
                    self.val
                }}
            }}
            "#
        }
    }

    /// Generate the `generated.rs` file for the given code generation results.
    ///
    /// ```rust,ignore
    /// use crate::ffi::bridging::*;
    /// use crate::types::*;
    ///
    /// pub trait MyModuleSpec {
    ///     fn multiply(a: f64, b: f64) -> f64;
    /// }
    /// ```
    pub fn generated_rs(codegen_res: &Vec<CodegenResult>) -> String {
        let spec_codes = codegen_res
            .iter()
            .map(|res| res.spec_code.clone())
            .collect::<Vec<_>>();

        let type_impls = codegen_res
            .iter()
            .map(|res| res.rs_type_impls.iter().map(|(_, v)| v.clone()))
            .flatten()
            .collect::<Vec<_>>();

        formatdoc! {
            r#"
            #[rustfmt::skip]
            use crate::ffi::bridging::*;
            use crate::types::*;

            {spec_codes}

            {type_impls}"#,
            type_impls = type_impls.join("\n\n"),
            spec_codes = spec_codes.join("\n\n"),
        }
    }

    fn cxx_bridging_impl(codegen_res: &Vec<CodegenResult>) -> Vec<String> {
        codegen_res
            .iter()
            .map(|res| res.rs_cxx_bridge.func_impls.join("\n\n"))
            .collect::<Vec<_>>()
    }

    pub fn alias_struct_def(name: &String, alias: &Alias) -> Result<String, anyhow::Error> {
        if alias.r#type != "ObjectTypeAnnotation" {
            return Err(anyhow::anyhow!(
                "Alias type should be ObjectTypeAnnotation, but got {}",
                alias.r#type
            ));
        }

        let mut struct_defs = vec![];

        // Example:
        // ```
        // foo: String,
        // bar: f64,
        // baz: bool,
        // ```
        let props = alias
            .properties
            .iter()
            .map(|property| -> Result<String, anyhow::Error> {
                Ok(format!(
                    "{}: {},",
                    property.name,
                    property.type_annotation.as_rs_bridge_type()?.0
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;

        alias
            .properties
            .iter()
            .try_for_each(|property| -> Result<(), anyhow::Error> {
                if let TypeAnnotation::NullableTypeAnnotation { type_annotation } =
                    &*property.type_annotation
                {
                    let name = property.type_annotation.as_rs_bridge_type()?.0;
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
            name = name,
            props = indent_str(props.join("\n"), 4),
        };

        struct_defs.push(struct_def);

        Ok(struct_defs.join("\n\n"))
    }

    pub fn alias_default_impl(
        name: &String,
        alias_schema: &Alias,
    ) -> Result<String, anyhow::Error> {
        let mut default_impls = vec![];

        let props_with_default_val = alias_schema
            .properties
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
            name = name,
            props = indent_str(props_with_default_val.join(",\n"), 12),
        };

        alias_schema
            .properties
            .iter()
            .try_for_each(|property| -> Result<(), anyhow::Error> {
                if let TypeAnnotation::NullableTypeAnnotation { type_annotation } =
                    &*property.type_annotation
                {
                    let nullable_type = property.type_annotation.as_rs_bridge_type()?.0;
                    let rs_impl_type = type_annotation.as_rs_impl_type()?.0;
                    let default_val = type_annotation.as_rs_default_val()?;

                    let default_impl = formatdoc! {
                        r#"
                        impl Default for {nullable_type} {{
                            fn default() -> Self {{
                                {nullable_type} {{
                                    null: true,
                                    val: {default_val}
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

    pub fn enum_default_impl(name: &String, enum_schema: &Enum) -> Result<String, anyhow::Error> {
        let first_member = enum_schema
            .members
            .as_ref()
            .expect("Enum members are required")
            .get(0)
            .expect("Enum members are required")
            .name
            .clone();

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
