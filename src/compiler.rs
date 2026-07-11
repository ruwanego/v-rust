use frontend::{lex, sema};
use std::fmt::Write as _;
use std::path::Path;

pub fn compile_file(input: &Path, output: &Path) -> Result<(), String> {
    if !input.exists() {
        return Err(format!("Error: File {} not found.", input.display()));
    }

    let source_code =
        std::fs::read_to_string(input).map_err(|e| format!("Error reading file: {e}"))?;

    let tokens = lex::tokenize(&source_code, input)?;

    let program = frontend::parse_tokens(&source_code, input, tokens)?;

    let mut analyzer = sema::SemanticAnalyzer::new();
    let checked_program = match analyzer.analyze(&program) {
        Ok(checked_program) => checked_program,
        Err(errors) => {
            let mut message = String::new();
            for err in errors {
                let (line, column) = frontend::source::line_column(&source_code, err.span.start);
                writeln!(
                    &mut message,
                    "Semantic error at {}:{line}:{column}: {err}",
                    input.display()
                )
                .map_err(|e| format!("Failed to render semantic error: {e}"))?;
            }
            return Err(message);
        }
    };

    #[cfg(feature = "codegen")]
    {
        use codegen_traits::CodegenBackend as _;

        codegen_llvm::LlvmBackend.compile(&checked_program, output).map_err(|e| e.to_string())?;
    }

    #[cfg(not(feature = "codegen"))]
    {
        let _ = output;
        let _ = checked_program;
    }

    Ok(())
}
