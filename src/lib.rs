#![forbid(unsafe_code)]

#[cfg(feature = "codegen")]
pub mod codegen;
pub mod lex;
pub mod parse;
pub mod sema;
