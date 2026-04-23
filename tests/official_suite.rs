use assert_cmd::Command;
use libtest_mimic::{Arguments, Trial};
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use walkdir::WalkDir;

fn main() {
    let args = Arguments::from_args();
    let repo_dir = PathBuf::from("tests/v_official_repo");

    if !repo_dir.exists() {
        println!("Downloading official V tests (RED GREEN REFACTOR)...");
        StdCommand::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                "https://github.com/vlang/v",
                repo_dir.to_str().unwrap(),
            ])
            .status()
            .expect("Failed to clone official V repository");
    }

    let mut tests = Vec::new();
    let test_dirs = vec![repo_dir.join("vlib"), repo_dir.join("cmd/v/tests")];

    for dir in test_dirs {
        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("v")
                && path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .ends_with("_test.v")
            {
                let test_name = path.strip_prefix(&repo_dir).unwrap().display().to_string();
                let path_clone = path.to_path_buf();

                tests.push(Trial::test(test_name, move || run_v_test(&path_clone)));
            }
        }
    }

    libtest_mimic::run(&args, tests).exit();
}

fn run_v_test(v_file: &Path) -> Result<(), libtest_mimic::Failed> {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_bin = temp_dir.path().join("test_bin");

    let mut compiler_cmd = Command::cargo_bin("v-rust").unwrap();
    let compile_result = compiler_cmd.arg(v_file).arg("-o").arg(&output_bin).ok();

    if compile_result.is_err() {
        return Err(format!("Compilation Failed for {}", v_file.display()).into());
    }

    // We expect an executable to exist, but our dummy CLI doesn't make one yet.
    if !output_bin.exists() {
        return Err("Compiler did not produce an output binary".into());
    }

    Ok(())
}
