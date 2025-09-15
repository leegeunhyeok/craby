use craby_common::{
    constants::impl_mod_name,
    utils::string::{flat_case, snake_case},
};
use indoc::formatdoc;
use log::error;

use crate::{
    types::{
        schema::{FunctionSpec, Parameter, TypeAnnotation},
        types::CodegenResult,
    },
    utils::indent_str,
};

pub trait ToRsType {
    fn to_rs_type(&self) -> Result<String, anyhow::Error>;
}

pub trait ToSig {
    fn to_sig(&self) -> Result<String, anyhow::Error>;
}

pub trait ToExternType {
    fn to_extern_type(&self) -> Result<String, anyhow::Error>;
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

impl ToSig for FunctionSpec {
    fn to_sig(&self) -> Result<String, anyhow::Error> {
        match &*self.type_annotation {
            TypeAnnotation::FunctionTypeAnnotation {
                return_type_annotation,
                params,
            } => {
                let return_type = return_type_annotation.to_rs_type()?;
                let params_sig = params
                    .iter()
                    .map(|p| p.to_sig())
                    .collect::<Result<Vec<_>, _>>()
                    .map(|p| p.join(", "))?;

                let fn_name = snake_case(&self.name);
                let ret_annotation = if return_type == "()" {
                    String::new()
                } else {
                    format!(" -> {}", return_type)
                };

                Ok(format!(
                    "fn {}({}){}",
                    fn_name.to_string(),
                    params_sig,
                    ret_annotation
                ))
            }
            _ => unimplemented!("Unsupported type annotation for function: {}", self.name),
        }
    }
}

impl ToSig for Parameter {
    fn to_sig(&self) -> Result<String, anyhow::Error> {
        if let TypeAnnotation::ObjectTypeAnnotation { .. }
        | TypeAnnotation::GenericObjectTypeAnnotation { .. } = *self.type_annotation
        {
            error!("Object type is not supported for parameters");
            error!("Use defined type alias instead");
            unimplemented!();
        }

        if let TypeAnnotation::FunctionTypeAnnotation { .. } = *self.type_annotation {
            error!("Function type is not supported for parameters");
            unimplemented!();
        }

        let (type_annotation, is_nullable) = self.type_annotation.unwrap_nullable();
        let param_type = type_annotation.to_rs_type()?;

        let final_type = if self.optional && !is_nullable {
            format!("Option<{}>", param_type)
        } else if is_nullable || self.optional {
            if param_type.starts_with("Option<") {
                param_type
            } else {
                format!("Option<{}>", param_type)
            }
        } else {
            param_type
        };

        Ok(format!("{}: {}", self.name, final_type))
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
        .map(|res| format!("pub(crate) mod {};", res.impl_mod.clone()))
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

    let cxx_externs = cxx_bridging_extern(&codgen_res);
    let cxx_impls = cxx_bridging_impl(&codgen_res);

    formatdoc! {
        r#"
        use ffi::*;
        use crate::generated::*;
        {impl_mods}

        {cxx_extern}

        {cxx_impl}"#,
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
        .map(|res| format!("use crate::ffi::{}::*;", res.ffi_mod.clone()))
        .collect::<Vec<_>>();

    let spec_codes = codegen_res
        .iter()
        .map(|res| res.spec_code.clone())
        .collect::<Vec<_>>();

    format!("{}\n\n{}", use_mods.join("\n"), spec_codes.join("\n\n"))
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
