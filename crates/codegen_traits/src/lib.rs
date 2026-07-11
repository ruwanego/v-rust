//! Abstract backend contract. Backends lower a `CheckedProgram` and emit a
//! native artifact; they must not own V language rules the frontend can prove.

#![forbid(unsafe_code)]

use frontend::sema::CheckedProgram;
use std::fmt;
use std::path::Path;

/// Backend selection used by the driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    /// Optimized LLVM backend (Inkwell).
    Llvm,
    // Cranelift variant lands with crates/codegen_cranelift (migration step 5).
}

/// Structured backend diagnostic.
#[derive(Debug)]
pub struct BackendError {
    pub message: String,
}

impl BackendError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for BackendError {}

/// Lower a checked frontend program and emit a native executable.
pub trait CodegenBackend {
    /// Compile `program` into an executable at `output`.
    fn compile(&self, program: &CheckedProgram, output: &Path) -> Result<(), BackendError>;

    /// Optional text dump of backend IR for snapshot tests.
    fn dump_ir(&self, _program: &CheckedProgram) -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoopBackend;

    impl CodegenBackend for NoopBackend {
        fn compile(&self, _program: &CheckedProgram, _output: &Path) -> Result<(), BackendError> {
            Err(BackendError::new("noop backend cannot emit code"))
        }
    }

    #[test]
    fn backend_error_displays_message() {
        let program = CheckedProgram { functions: vec![] };
        let err = NoopBackend.compile(&program, Path::new("out")).unwrap_err();
        assert_eq!(err.to_string(), "noop backend cannot emit code");
    }

    #[test]
    fn dump_ir_defaults_to_none() {
        let program = CheckedProgram { functions: vec![] };
        assert!(NoopBackend.dump_ir(&program).is_none());
    }
}
