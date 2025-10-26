#[macro_use]
pub mod macros;

/// This module provides the prelude for Craby Modules.
pub mod prelude {
    pub use crate::context::*;
    pub use crate::types::*;
    pub use craby_macro::craby_module;
}

pub mod context;
pub mod types;

// craby_marco crate
pub use craby_macro;
