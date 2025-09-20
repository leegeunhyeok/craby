use std::collections::BTreeMap;

use crate::platform::{cxx::CxxMethod, rust::RsCxxBridge};

pub struct CodegenResult {
    /// Module name from Schema (TurboModule name)
    pub module_name: String,
    /// Module name for the Rust impl module
    pub impl_mod: String,
    /// Code for the spec trait
    pub spec_code: String,
    /// Code for the snippet of the current module's spec
    pub impl_code: String,
    /// cxx bridging function signatures
    pub rs_cxx_bridge: RsCxxBridge,
    /// Rust type implementations
    pub rs_type_impls: BTreeMap<String, String>,
    /// cxx implementations
    pub cxx_methods: Vec<CxxMethod>,
    /// cxx bridging templates
    pub cxx_bridging_templates: Vec<String>,
}
