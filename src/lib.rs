#![forbid(unsafe_code)]

#[cfg(feature = "codegen")]
pub mod codegen;
pub mod compiler;
pub mod lex;
pub mod parse;
pub mod sema;
