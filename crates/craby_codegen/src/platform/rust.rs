use craby_common::constants::impl_mod_name;
use indoc::formatdoc;

use crate::{types::schema::CxxFunction, utils::indent_str};

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

pub fn ffi_rs(mod_names: &Vec<String>, cxx_functions: &Vec<CxxFunction>) -> String {
    let impl_mods = mod_names
        .iter()
        .map(|mod_name| format!("use crate::{}::*;", impl_mod_name(mod_name)))
        .collect::<Vec<String>>();

    let cxx_extern_functions = cxx_functions
        .iter()
        .map(|func| func.extern_func.clone())
        .collect::<Vec<String>>();

    let cxx_impl_functions = cxx_functions
        .iter()
        .map(|func| func.impl_func.clone())
        .collect::<Vec<String>>();

    formatdoc! {
        r#"
        use ffi::*;
        use crate::generated::*;
        {impl_mods}

        #[cxx::bridge(namespace = "craby::ffi")]
        pub mod ffi {{
            {type_defs}

            extern "Rust" {{
        {cxx_extern_functions}
            }}
        }}

        {cxx_impl_functions}"#,
        type_defs = "", // TODO
        impl_mods = impl_mods.join("\n"),
        cxx_extern_functions = indent_str(cxx_extern_functions.join("\n\n"), 8),
        cxx_impl_functions = cxx_impl_functions.join("\n\n"),
    }
}
