use craby_common::utils::string::{flat_case, pascal_case, snake_case};
use indoc::formatdoc;

use crate::{
    types::{
        schema::{FunctionSpec, TypeAnnotation},
        types::CodegenResult,
    },
    utils::indent_str,
};

pub trait ToRsType {
    /// Returns the Rust type for the given `TypeAnnotation`.
    fn to_rs_type(&self) -> Result<String, anyhow::Error>;
}

pub trait ToExternType {
    /// Returns the Rust type for the given `TypeAnnotation` that is used in the cxx extern function.
    fn to_extern_type(&self) -> Result<String, anyhow::Error>;
}

pub trait ToCxxBridge {
    /// Returns the cxx(FFI) function declaration and implementation for the `FunctionSpec`.
    fn to_cxx_bridge(&self, mod_name: &String) -> Result<CxxBridge, anyhow::Error>;
}

#[derive(Debug, Clone)]
pub struct CxxBridge {
    /// The extern function declaration.
    ///
    /// **Example**
    ///
    /// ```rust,ignore
    /// #[cxx_name = "myFunc"]
    /// fn myFunc(arg1: Foo, arg2: Bar) -> Baz;
    /// ```
    pub extern_func: String,
    /// The implementation function of the extern function.
    ///
    /// **Example**
    ///
    /// ```rust,ignore
    /// fn myFunc(arg1: Foo, arg2: Bar) -> Baz {
    ///   MyModule::my_func(arg1, arg2)
    /// }
    /// ```
    pub impl_func: String,
}

impl ToRsType for TypeAnnotation {
    fn to_rs_type(&self) -> Result<String, anyhow::Error> {
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
                format!("Vec<{}>", element_type.to_rs_type()?)
            }

            // Type alias
            TypeAnnotation::TypeAliasTypeAnnotation { name } => name.clone(),

            // Enum
            TypeAnnotation::EnumDeclaration { name, .. } => name.clone(),

            // Promise type
            TypeAnnotation::PromiseTypeAnnotation { element_type } => {
                format!("Result<{}, anyhow::Error>", element_type.to_rs_type()?)
            }

            // Void type
            TypeAnnotation::VoidTypeAnnotation => "()".to_string(),

            _ => {
                return Err(anyhow::anyhow!("Unsupported type annotation: {:?}", self));
            }
        };

        Ok(rs_type)
    }
}

impl ToExternType for TypeAnnotation {
    fn to_extern_type(&self) -> Result<String, anyhow::Error> {
        let extern_type = match self {
            TypeAnnotation::PromiseTypeAnnotation { element_type } => {
                format!("Result<{}>", element_type.to_rs_type()?)
            }
            _ => self.to_rs_type()?,
        };

        Ok(extern_type)
    }
}

impl ToCxxBridge for FunctionSpec {
    fn to_cxx_bridge(&self, mod_name: &String) -> Result<CxxBridge, anyhow::Error> {
        match &*self.type_annotation {
            TypeAnnotation::FunctionTypeAnnotation {
                return_type_annotation,
                params,
            } => {
                let ret_type = return_type_annotation.to_rs_type()?;
                let ret_extern_type = return_type_annotation.to_extern_type()?.to_string();
                let params_sig = params
                    .iter()
                    .map(|param| param.to_sig())
                    .collect::<Result<Vec<_>, _>>()
                    .map(|param| param.join(", "))?;

                let impl_name = pascal_case(mod_name);
                let mod_name = snake_case(mod_name);
                let fn_name = snake_case(&self.name);
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
                    orig_fn_name = self.name,
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

                Ok(CxxBridge {
                    extern_func,
                    impl_func,
                })
            }
            _ => unimplemented!("Unsupported type annotation for function: {}", self.name),
        }
    }
}

fn cxx_bridging_extern(codegen_res: &Vec<CodegenResult>) -> Vec<String> {
    codegen_res
        .iter()
        .map(|res| {
            let flat_name = flat_case(&res.module_name);
            let snake_name = snake_case(&res.module_name);
            let cxx_extern = res
                .cxx_bridges
                .iter()
                .map(|bridge| bridge.extern_func.clone())
                .collect::<Vec<_>>();

            formatdoc! {
                r#"
                #[cxx::bridge(namespace = "craby::{flat_name}")]
                pub mod {snake_name} {{
                    // Type definitions
                {type_defs}

                    extern "Rust" {{
                {cxx_extern}
                    }}
                }}"#,
                flat_name = flat_name,
                snake_name = snake_name,
                type_defs = indent_str("// N/A".to_string(), 4), // TODO
                cxx_extern = indent_str(cxx_extern.join("\n\n"), 8),
            }
        })
        .collect::<Vec<_>>()
}

pub mod template {
    use craby_common::constants::impl_mod_name;
    use indoc::formatdoc;

    use crate::{platform::rust::cxx_bridging_extern, types::types::CodegenResult};

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
            .map(|res| {
                res.cxx_bridges
                    .iter()
                    .map(|bridge| bridge.impl_func.clone())
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<_>>()
    }
}
