use crate::platform::{cxx::CxxMethod, rust::CxxBridge};

pub struct CodegenResult {
    /// Module name from Schema (TurboModule name)
    pub module_name: String,
    /// Module name for the Rust ffi module
    pub ffi_mod: String,
    /// Module name for the Rust impl module
    pub impl_mod: String,
    /// Code for the spec trait
    pub spec_code: String,
    /// Code for the snippet of the current module's spec
    pub impl_code: String,
    /// cxx bridging function signatures
    pub cxx_bridges: Vec<CxxBridge>,
    /// cxx implementations
    pub cxx_methods: Vec<CxxMethod>,
}
