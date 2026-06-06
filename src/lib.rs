#![forbid(unsafe_code)]

#[cfg(feature = "codegen")]
pub mod codegen;
pub mod compiler;
pub use frontend;
