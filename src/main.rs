#![forbid(unsafe_code)]

use clap::{Args, CommandFactory, Parser, Subcommand};
use std::ffi::OsStr;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use std::process::exit;
use v_rust::{lex, parse, sema};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(author, version, about = "V Compiler written in Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// The .v file to compile
    input: Option<PathBuf>,

    /// Output binary name
    #[arg(short, long, default_value = "a.out")]
    output: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Some(Command::Test(args)) => run_test_command(args),
        None => {
            let Some(input) = &cli.input else {
                let mut command = Cli::command();
                command.print_help().unwrap_or_else(|e| eprintln!("{e}"));
                eprintln!();
                exit(2);
            };
            compile_file(input, &cli.output)
        }
    };

    if let Err(err) = result {
        eprintln!("{err}");
        exit(1);
    }
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Recursively run V _test.v files.
    Test(TestArgs),
}

#[derive(Args, Debug)]
struct TestArgs {
    /// Files or directories to test.
    paths: Vec<PathBuf>,
}

fn compile_file(input: &Path, output: &Path) -> Result<(), String> {
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
        let codegen = v_rust::codegen::CodeGen::new(&context, "v_module");
        codegen.generate(&program);

        let obj_dir =
            tempfile::tempdir().map_err(|e| format!("Failed to create object temp dir: {e}"))?;
        let obj_path = obj_dir.path().join("output.o");
        if let Err(e) = codegen.write_obj(&obj_path) {
            return Err(format!("Codegen error: {e}"));
        }

        let linker_output = ProcessCommand::new("clang")
            .arg(&obj_path)
            .arg("-o")
            .arg(output)
            .output()
            .map_err(|e| format!("Failed to execute linker: {e}"))?;

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

fn run_test_command(args: &TestArgs) -> Result<(), String> {
    let default_paths = [PathBuf::from(".")];
    let paths = if args.paths.is_empty() {
        default_paths.as_slice()
    } else {
        args.paths.as_slice()
    };
    let test_files = discover_test_files(paths)?;

    if test_files.is_empty() {
        println!("No V _test.v files found.");
        return Ok(());
    }

    println!("Running {} V _test.v file(s)", test_files.len());

    let mut failures = Vec::new();
    for test_file in &test_files {
        match run_test_file(test_file) {
            Ok(()) => println!("ok {}", test_file.display()),
            Err(err) => failures.push(TestFailure {
                path: test_file.clone(),
                message: err,
            }),
        }
    }

    if failures.is_empty() {
        println!(
            "Summary for V _test.v files: {} passed, {} total.",
            test_files.len(),
            test_files.len()
        );
        return Ok(());
    }

    for failure in &failures {
        eprintln!("---- {} ----", failure.path.display());
        eprintln!("{}", failure.message.trim_end());
    }

    Err(format!(
        "Summary for V _test.v files: {} failed, {} passed, {} total.",
        failures.len(),
        test_files.len() - failures.len(),
        test_files.len()
    ))
}

#[derive(Debug)]
struct TestFailure {
    path: PathBuf,
    message: String,
}

fn discover_test_files(paths: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    let mut test_files = Vec::new();

    for path in paths {
        if path.is_file() {
            if is_v_test_file(path) && !is_under_testdata(path) {
                test_files.push(path.clone());
            }
            continue;
        }

        if !path.is_dir() {
            return Err(format!("Test path {} does not exist.", path.display()));
        }

        for entry in WalkDir::new(path)
            .into_iter()
            .filter_entry(|entry| !is_testdata_dir(entry.path()))
            .filter_map(Result::ok)
        {
            let candidate = entry.path();
            if candidate.is_file() && is_v_test_file(candidate) {
                test_files.push(candidate.to_path_buf());
            }
        }
    }

    test_files.sort();
    test_files.dedup();

    Ok(test_files)
}

fn is_v_test_file(path: &Path) -> bool {
    path.extension().and_then(OsStr::to_str) == Some("v")
        && path
            .file_name()
            .and_then(OsStr::to_str)
            .is_some_and(|name| name.ends_with("_test.v"))
}

fn is_testdata_dir(path: &Path) -> bool {
    path.file_name() == Some(OsStr::new("testdata"))
}

fn is_under_testdata(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str() == OsStr::new("testdata"))
}

fn run_test_file(test_file: &Path) -> Result<(), String> {
    let temp_dir =
        tempfile::tempdir().map_err(|e| format!("Failed to create test temp dir: {e}"))?;
    let test_binary = temp_dir
        .path()
        .join(format!("test_bin{}", std::env::consts::EXE_SUFFIX));

    compile_file(test_file, &test_binary).map_err(|e| format!("Compilation failed:\n{e}"))?;

    if !test_binary.exists() {
        return Err("Compiler did not produce an output binary.".to_string());
    }

    let output = ProcessCommand::new(&test_binary)
        .output()
        .map_err(|e| format!("Failed to execute test binary: {e}"))?;

    if output.status.success() {
        return Ok(());
    }

    Err(format!(
        "Test binary exited with status {}.\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
}
