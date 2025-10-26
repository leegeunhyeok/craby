#[cfg(feature = "artifact")]
pub mod cargo;

#[cfg(feature = "artifact")]
pub mod constants;

#[cfg(feature = "artifact")]
pub mod platform;

#[cfg(feature = "cxx")]
mod cxx;

#[cfg(feature = "cxx")]
pub use cxx::setup;
