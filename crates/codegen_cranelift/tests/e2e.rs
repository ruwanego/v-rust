//! End-to-end proof: V source -> Cranelift object -> platform link -> run.

use codegen_traits::CodegenBackend;
use frontend::{lex, sema};
use std::path::Path;
use std::process::Command;

fn compile_and_run(source: &str) -> String {
    let input = Path::new("e2e.v");
    let tokens = lex::tokenize(source, input).expect("lex");
    let program = frontend::parse_tokens(source, input, tokens).expect("parse");
    let checked =
        sema::SemanticAnalyzer::new().analyze(&program).expect("sema should accept program");

    let dir = tempfile::tempdir().expect("tempdir");
    let exe = dir.path().join(if cfg!(windows) { "e2e.exe" } else { "e2e" });
    codegen_cranelift::CraneliftBackend.compile(&checked, &exe).expect("backend compile");

    let out = Command::new(&exe).output().expect("run compiled binary");
    assert!(out.status.success(), "binary exited with {:?}", out.status);
    // Windows CRT stdout is text-mode; normalize line endings for comparison.
    String::from_utf8(out.stdout).expect("utf8 stdout").replace("\r\n", "\n")
}

#[test]
fn compiles_links_and_runs_v_program() {
    let source = "fn main() {\n\
                  println('hello from cranelift')\n\
                  mut value := 5\n\
                  value = value + 2\n\
                  println(value * 10 - 4)\n\
                  println(true)\n\
                  println(1 < 2)\n\
                  }\n";
    let stdout = compile_and_run(source);
    assert_eq!(stdout, "hello from cranelift\n66\n1\n1\n");
}
