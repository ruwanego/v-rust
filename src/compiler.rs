use crate::{lex, parse, sema};
use std::fmt::Write as _;
use std::path::Path;

#[cfg(feature = "codegen")]
use std::{
    env,
    io::ErrorKind,
    process::{Command as ProcessCommand, Output},
};

pub fn compile_file(input: &Path, output: &Path) -> Result<(), String> {
    if !input.exists() {
        return Err(format!("Error: File {} not found.", input.display()));
    }

    let source_code =
        std::fs::read_to_string(input).map_err(|e| format!("Error reading file: {e}"))?;

    let tokens: Result<Vec<_>, _> = <lex::Token as logos::Logos>::lexer(&source_code).collect();
    let Ok(tokens) = tokens else {
        return Err("Lexer error".to_string());
    };

    let program = match chumsky::Parser::parse(&parse::parser(), tokens) {
        Ok(p) => p,
        Err(e) => return Err(format!("Parser error: {e:?}")),
    };

    let mut analyzer = sema::SemanticAnalyzer::new();
    if let Err(errors) = analyzer.analyze(&program) {
        let mut message = String::new();
        for err in errors {
            writeln!(&mut message, "Semantic error: {}", err.message)
                .map_err(|e| format!("Failed to render semantic error: {e}"))?;
        }
        return Err(message);
    }

    #[cfg(feature = "codegen")]
    {
        use inkwell::context::Context;

        let context = Context::create();
        let codegen = crate::codegen::CodeGen::new(&context, "v_module");
        codegen.generate(&program);

        let obj_dir =
            tempfile::tempdir().map_err(|e| format!("Failed to create object temp dir: {e}"))?;
        let obj_path = obj_dir.path().join("output.o");
        if let Err(e) = codegen.write_obj(&obj_path) {
            return Err(format!("Codegen error: {e}"));
        }

        let linker_output = link_object(&obj_path, output)?;

        if !linker_output.status.success() {
            return Err(format!(
                "Linker failed:\n{}",
                String::from_utf8_lossy(&linker_output.stderr)
            ));
        }
    }

    #[cfg(not(feature = "codegen"))]
    {
        let _ = output;
    }

    Ok(())
}

#[cfg(feature = "codegen")]
fn link_object(obj_path: &Path, output: &Path) -> Result<Output, String> {
    let configured_linker = env::var("CLANG")
        .ok()
        .filter(|value| !value.trim().is_empty());
    let candidates = configured_linker
        .iter()
        .map(String::as_str)
        .chain(["clang", "clang-15"]);
    let mut missing_linkers = Vec::new();

    for linker in candidates {
        match ProcessCommand::new(linker)
            .arg(obj_path)
            .arg("-o")
            .arg(output)
            .output()
        {
            Ok(output) => return Ok(output),
            Err(error) if error.kind() == ErrorKind::NotFound => {
                missing_linkers.push(linker.to_string());
            }
            Err(error) => {
                return Err(format!("Failed to execute linker `{linker}`: {error}"));
            }
        }
    }

    Err(format!(
        "Failed to execute linker. Tried: {}",
        missing_linkers.join(", ")
    ))
}
