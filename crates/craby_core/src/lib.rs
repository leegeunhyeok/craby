pub mod interop;
pub(crate) mod type_bindings;

#[cfg(feature = "android")]
pub mod android {
    pub use crate::interop::android as interop;
    pub use crate::type_bindings::android as types;
}

#[cfg(feature = "ios")]
pub mod ios {
    pub use crate::interop::ios as interop;
    pub use crate::type_bindings::ios as types;
}

// Third party crates
#[cfg(feature = "android")]
pub use jni;
