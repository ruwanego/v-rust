use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use v_rust::compiler::compile_file;
use walkdir::WalkDir;

pub(super) fn run(paths: &[PathBuf]) -> Result<(), String> {
    let default_paths = [PathBuf::from(".")];
    let paths = if paths.is_empty() { default_paths.as_slice() } else { paths };
    let test_files = discover_test_files(paths)?;

    if test_files.is_empty() {
        println!("No V _test.v files found.");
        return Ok(());
    }

    let total = test_files.len();
    println!("Running {total} V _test.v file(s)");

    let mut failures = Vec::new();
    for (index, test_file) in test_files.iter().enumerate() {
        let ordinal = index + 1;
        let test_name = test_file.display();
        println!("[{ordinal}/{total}] RUN {test_name}");

        match run_test_file(test_file) {
            Ok(()) => println!("[{ordinal}/{total}] PASS {test_name}"),
            Err(err) => {
                eprintln!("[{ordinal}/{total}] FAIL {test_name}");
                failures.push(TestFailure { ordinal, path: test_file.clone(), message: err });
            }
        }
    }

    if failures.is_empty() {
        println!("Summary for V _test.v files: {total} passed, {total} total.");
        return Ok(());
    }

    eprintln!("Failed V _test.v files in run order:");
    for failure in &failures {
        let ordinal = failure.ordinal;
        let test_name = failure.path.display();
        eprintln!("[{ordinal}/{total}] {test_name}");
    }
    eprintln!();

    for failure in &failures {
        let ordinal = failure.ordinal;
        let test_name = failure.path.display();
        eprintln!("---- [{ordinal}/{total}] {test_name} ----");
        eprintln!("{}", failure.message.trim_end());
    }

    Err(format!(
        "Summary for V _test.v files: {} failed, {} passed, {total} total.",
        failures.len(),
        test_files.len() - failures.len()
    ))
}

#[derive(Debug)]
struct TestFailure {
    ordinal: usize,
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
        && path.file_name().and_then(OsStr::to_str).is_some_and(|name| name.ends_with("_test.v"))
}

fn is_testdata_dir(path: &Path) -> bool {
    path.file_name() == Some(OsStr::new("testdata"))
}

fn is_under_testdata(path: &Path) -> bool {
    path.components().any(|component| component.as_os_str() == OsStr::new("testdata"))
}

fn run_test_file(test_file: &Path) -> Result<(), String> {
    let temp_dir =
        tempfile::tempdir().map_err(|e| format!("Failed to create test temp dir: {e}"))?;
    let test_binary = temp_dir.path().join(format!("test_bin{}", std::env::consts::EXE_SUFFIX));

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
