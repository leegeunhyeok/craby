use craby_common::utils::string::{flat_case, pascal_case, snake_case};
use indoc::formatdoc;
use template::alias_struct_def;

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
pub struct RsExternType(pub String);

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

            // Void type
            TypeAnnotation::VoidTypeAnnotation => "()".to_string(),

            _ => {
                return Err(anyhow::anyhow!("Unsupported type annotation: {:?}", self));
            }
        };

        Ok(RsType(rs_type))
    }

    /// Returns the Rust type for the given `TypeAnnotation` that is used in the cxx extern function.
    pub fn as_rs_extern_type(&self) -> Result<RsExternType, anyhow::Error> {
        let extern_type = match self {
            TypeAnnotation::PromiseTypeAnnotation { element_type } => {
                format!("Result<{}>", element_type.as_rs_type()?.0)
            }
            _ => self.as_rs_type()?.0,
        };

        Ok(RsExternType(extern_type))
    }
}

impl Schema {
    /// Returns the Rust cxx bridging function declaration and implementation for the `FunctionSpec`.
    pub fn as_rs_cxx_bridge(&self) -> Result<RsCxxBridge, anyhow::Error> {
        let mut func_extern_sigs = vec![];
        let mut func_impls = vec![];
        let mut struct_defs = vec![];
        let mut enum_defs = vec![];

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
                        let ret_type = return_type_annotation.as_rs_type()?.0;
                        let ret_extern_type = return_type_annotation.as_rs_extern_type()?.0;
                        let params_sig = params
                            .iter()
                            .map(|param| param.as_sig())
                            .collect::<Result<Vec<_>, _>>()
                            .map(|params| params.join(", "))?;

                        let impl_name = pascal_case(&self.module_name);
                        let mod_name = snake_case(&self.module_name);
                        let fn_name = snake_case(&spec.name);
                        let fn_args = params.iter().map(|p| p.name.clone()).collect::<Vec<_>>();
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

                        let impl_func = formatdoc! {
                            r#"
                            fn {prefixed_fn_name}({params_sig}){ret} {{
                                {impl_name}::{fn_name}({fn_args})
                            }}"#,
                            params_sig = params_sig,
                            ret = ret_annotation,
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
                            "Unsupported type annotation for function: {}",
                            spec.name
                        ))
                    }
                }
            })?;

        // Collect alias types (struct)
        self.alias_map.iter().try_for_each(
            |(name, elias_schema)| -> Result<(), anyhow::Error> {
                struct_defs.push(alias_struct_def(name, elias_schema)?);
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
}

fn cxx_bridging_extern(codegen_res: &Vec<CodegenResult>) -> Vec<String> {
    codegen_res
        .iter()
        .map(|res| {
            let flat_name = flat_case(&res.module_name);
            let snake_name = snake_case(&res.module_name);
            let cxx_extern = res.rs_cxx_bridge.func_extern_sigs.join("\n\n");
            let struct_defs = res.rs_cxx_bridge.struct_defs.join("\n\n");
            let enum_defs = res.rs_cxx_bridge.enum_defs.join("\n\n");

            formatdoc! {
                r#"
                #[cxx::bridge(namespace = "craby::{flat_name}")]
                pub mod {snake_name} {{
                    // Type definitions
                {struct_defs}

                {enum_defs}

                    extern "Rust" {{
                {cxx_extern}
                    }}
                }}"#,
                flat_name = flat_name,
                snake_name = snake_name,
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
        types::{schema::Alias, types::CodegenResult},
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
            pub(crate) mod ffi;
            pub(crate) mod generated;
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
    /// pub mod my_module {
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

        let ffi_mods = codgen_res
            .iter()
            .map(|res| format!("use {}::*;", res.ffi_mod))
            .collect::<Vec<_>>();

        let cxx_externs = cxx_bridging_extern(&codgen_res);
        let cxx_impls = cxx_bridging_impl(&codgen_res);

        formatdoc! {
            r#"
            {ffi_mods}
            {impl_mods}
            use crate::generated::*;

            {cxx_extern}

            {cxx_impl}"#,
            ffi_mods = ffi_mods.join("\n"),
            impl_mods = impl_mods.join("\n"),
            cxx_extern = cxx_externs.join("\n\n"),
            cxx_impl = cxx_impls.join("\n\n"),
        }
    }

    /// Generate the `generated.rs` file for the given code generation results.
    ///
    /// ```rust,ignore
    /// use crate::ffi::my_module::*;
    ///
    /// pub trait MyModuleSpec {
    ///     fn multiply(a: f64, b: f64) -> f64;
    /// }
    /// ```
    pub fn generated_rs(codegen_res: &Vec<CodegenResult>) -> String {
        let use_mods = codegen_res
            .iter()
            .map(|res| format!("use crate::ffi::{}::*;", res.ffi_mod))
            .collect::<Vec<_>>();

        let spec_codes = codegen_res
            .iter()
            .map(|res| res.spec_code.clone())
            .collect::<Vec<_>>();

        format!("{}\n\n{}", use_mods.join("\n"), spec_codes.join("\n\n"))
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
                    property.type_annotation.as_rs_extern_type()?.0
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let struct_def = formatdoc! {
            r#"
                struct {name} {{
                {props}
                }}"#,
            name = name,
            props = indent_str(props.join("\n"), 4),
        };

        Ok(struct_def)
    }
}
