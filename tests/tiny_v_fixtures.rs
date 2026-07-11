mod tiny_common;

use libtest_mimic::{Arguments, Failed, Trial};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;

fn main() {
    let args = Arguments::from_args();
    let mut tests = Vec::new();

    for fixture in collect_fixtures("tests/fixtures/tiny/pass") {
        let name = fixture_name("tiny pass", &fixture);
        tests.push(Trial::test(name, move || run_pass_fixture(&fixture)));
    }

    for fixture in collect_fixtures("tests/fixtures/tiny/fail") {
        let name = fixture_name("tiny fail", &fixture);
        tests.push(Trial::test(name, move || run_fail_fixture(&fixture)));
    }

    libtest_mimic::run(&args, tests).exit();
}

fn collect_fixtures(dir: &str) -> Vec<PathBuf> {
    let mut fixtures = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("failed to read fixture directory {dir}: {e}"))
        .map(|entry| entry.expect("fixture directory entry should be readable").path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("v"))
        .collect::<Vec<_>>();
    fixtures.sort();
    fixtures
}

fn fixture_name(kind: &str, fixture: &Path) -> String {
    let name = fixture
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .expect("fixture should have UTF-8 file name");
    format!("{kind}::{name}")
}

fn run_pass_fixture(fixture: &Path) -> Result<(), Failed> {
    let temp_dir = tempfile::tempdir().map_err(|e| format!("failed to create temp dir: {e}"))?;
    let binary = temp_dir.path().join(format!("fixture{}", std::env::consts::EXE_SUFFIX));

    let compile_output =
        tiny_common::v_rust_command().arg(fixture).arg("-o").arg(&binary).output().unwrap();

    if !compile_output.status.success() {
        return Err(render_process_failure("compile", fixture, &compile_output).into());
    }

    if !binary.exists() {
        return Err(format!("compiler did not produce binary {}", binary.display()).into());
    }

    let run_output = StdCommand::new(&binary).output().unwrap();
    if !run_output.status.success() {
        return Err(render_process_failure("run", fixture, &run_output).into());
    }

    // Normalize CRLF so fixtures behave identically on Windows text-mode
    // stdout and git autocrlf checkouts.
    let expected_stdout = fs::read_to_string(fixture.with_extension("stdout"))
        .map_err(|e| format!("failed to read expected stdout for {}: {e}", fixture.display()))?
        .replace("\r\n", "\n");
    let actual_stdout = String::from_utf8_lossy(&run_output.stdout).replace("\r\n", "\n");
    if actual_stdout != expected_stdout {
        return Err(format!(
            "stdout mismatch for {}\nexpected:\n{}\nactual:\n{}",
            fixture.display(),
            expected_stdout,
            actual_stdout
        )
        .into());
    }

    Ok(())
}

fn run_fail_fixture(fixture: &Path) -> Result<(), Failed> {
    let output = tiny_common::v_rust_command().arg(fixture).output().unwrap();

    if output.status.success() {
        return Err(
            format!("fixture {} compiled successfully but should fail", fixture.display()).into()
        );
    }

    let expected_error = fs::read_to_string(fixture.with_extension("stderr"))
        .map_err(|e| format!("failed to read expected stderr for {}: {e}", fixture.display()))?;
    // Normalize Windows path separators so .stderr expectations stay portable.
    let actual_error = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
    .replace('\\', "/");

    if !actual_error.contains(expected_error.trim()) {
        return Err(format!(
            "error mismatch for {}\nexpected substring:\n{}\nactual:\n{}",
            fixture.display(),
            expected_error,
            actual_error
        )
        .into());
    }

    Ok(())
}

fn render_process_failure(stage: &str, fixture: &Path, output: &std::process::Output) -> String {
    format!(
        "{stage} failed for {} with status {}\nstdout:\n{}\nstderr:\n{}",
        fixture.display(),
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}
