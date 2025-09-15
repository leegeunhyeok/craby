use craby_common::{constants::impl_mod_name, utils::string::snake_case};
use indoc::formatdoc;
use log::error;

use crate::{
    types::{
        schema::{FunctionSpec, Parameter, TypeAnnotation},
        types::ToType,
    },
    utils::indent_str,
};

use super::cxx::CxxBridge;

pub trait ToSig {
    fn to_sig(&self) -> Result<String, anyhow::Error>;
}

pub trait ToExternType {
    fn to_extern_type(&self) -> Result<String, anyhow::Error>;
}

impl ToSig for FunctionSpec {
    fn to_sig(&self) -> Result<String, anyhow::Error> {
        match &*self.type_annotation {
            TypeAnnotation::FunctionTypeAnnotation {
                return_type_annotation,
                params,
            } => {
                let return_type = return_type_annotation.to_type()?.to_string();
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
        let param_type = type_annotation.to_type()?.to_string();

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
        let r#type = self.to_type()?;
        Ok(r#type.to_string())
    }
}

/// Generate the lib.rs file for the given module names.
///
/// ```rust,ignore
/// use crate::generated::*;
///
/// pub(crate) mod generated;
/// pub(crate) mod ffi;
/// pub(crate) mod my_module_impl;
/// ```
pub fn lib_rs(impl_mods: &Vec<String>) -> String {
    let impl_mods = impl_mods
        .iter()
        .map(|name| format!("pub(crate) mod {};", impl_mod_name(name)))
        .collect::<Vec<String>>();

    formatdoc! {
        r#"
        pub(crate) mod ffi;
        pub(crate) mod generated;
        {impl_mods}"#,
        impl_mods = impl_mods.join("\n"),
    }
}

pub fn ffi_rs(mod_names: &Vec<String>, cxx_bridges: &Vec<CxxBridge>) -> String {
    let impl_mods = mod_names
        .iter()
        .map(|mod_name| format!("use crate::{}::*;", impl_mod_name(mod_name)))
        .collect::<Vec<_>>();

    let cxx_extern = cxx_bridges
        .iter()
        .map(|func| func.extern_func.clone())
        .collect::<Vec<_>>();

    let cxx_impl = cxx_bridges
        .iter()
        .map(|func| func.impl_func.clone())
        .collect::<Vec<_>>();

    formatdoc! {
        r#"
        use ffi::*;
        use crate::generated::*;
        {impl_mods}

        #[cxx::bridge(namespace = "craby::ffi")]
        pub mod ffi {{
            {type_defs}

            extern "Rust" {{
        {cxx_extern}
            }}
        }}

        {cxx_impl}"#,
        type_defs = "", // TODO
        impl_mods = impl_mods.join("\n"),
        cxx_extern = indent_str(cxx_extern.join("\n\n"), 8),
        cxx_impl = cxx_impl.join("\n\n"),
    }
}
