use clap::Parser;
use std::path::PathBuf;
use std::process::exit;

pub mod lex;
pub mod parse;
pub mod sema;
#[cfg(feature = "codegen")]
pub mod codegen;

#[derive(Parser, Debug)]
#[command(author, version, about = "V Compiler written in Rust", long_about = None)]
struct Args {
    /// The .v file to compile
    input: PathBuf,

    /// Output binary name
    #[arg(short, long, default_value = "a.out")]
    output: PathBuf,
}

fn main() {
    let args = Args::parse();

    // Right now, we just print what we are *supposed* to do.
    // If the input file doesn't exist, we fail.
    if !args.input.exists() {
        eprintln!("Error: File {} not found.", args.input.display());
        exit(1);
    }

    // 1. Lex
    let source_code = std::fs::read_to_string(&args.input).unwrap_or_else(|e| {
        eprintln!("Error reading file: {}", e);
        exit(1);
    });

    use logos::Logos;
    let tokens: Result<Vec<_>, _> = lex::Token::lexer(&source_code).collect();
    let tokens = match tokens {
        Ok(t) => t,
        Err(_) => {
            eprintln!("Lexer error");
            exit(1);
        }
    };

    // 2. Parse
    use chumsky::Parser;
    let program = match parse::parser().parse(tokens) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parser error: {:?}", e);
            exit(1);
        }
    };

    // 3. Sema
    let mut analyzer = sema::SemanticAnalyzer::new();
    if let Err(errors) = analyzer.analyze(&program) {
        for err in errors {
            eprintln!("Semantic error: {}", err.message);
        }
        exit(1);
    }

    // 4. Codegen (LLVM)
    #[cfg(feature = "codegen")]
    {
        use inkwell::context::Context;

        let context = Context::create();
        let codegen = codegen::CodeGen::new(&context, "v_module");
        codegen.generate(&program);

        let obj_path = std::env::temp_dir().join("output.o");
        if let Err(e) = codegen.write_obj(&obj_path) {
            eprintln!("Codegen error: {}", e);
            exit(1);
        }

        // 4. Link
        let output = std::process::Command::new("clang")
            .arg(obj_path.to_str().unwrap())
            .arg("-o")
            .arg(args.output.to_str().unwrap())
            .output()
            .unwrap_or_else(|e| {
                eprintln!("Failed to execute linker: {}", e);
                exit(1);
            });

        if !output.status.success() {
            eprintln!(
                "Linker failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
            exit(1);
        }
    }

    // Success!
}
