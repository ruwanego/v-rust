use assert_cmd::Command;
use libtest_mimic::Failed;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;

fn v_rust_command() -> Command {
    Command::cargo_bin("v-rust").expect("v-rust binary should be built by cargo test")
}

/// Pinned ref for the official V corpus so the acceptance suite is
/// deterministic. Bump by editing `tests/v_repo_ref.txt`, then delete
/// `tests/v_official_repo` and re-run.
const V_REPO_REF: &str = include_str!("../v_repo_ref.txt");

pub fn ensure_official_repo(repo_dir: &Path) {
    if repo_dir.exists() {
        return;
    }

    let repo_ref = V_REPO_REF.trim();
    println!("Downloading official V tests at {repo_ref} (RED GREEN REFACTOR)...");
    let status = StdCommand::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "--branch",
            repo_ref,
            "https://github.com/vlang/v",
            repo_dir.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to clone official V repository");

    assert!(status.success(), "Failed to clone official V repository");
}

pub fn official_repo_dir() -> PathBuf {
    PathBuf::from("tests/v_official_repo")
}

pub fn run_v_rust_test(path: &Path) -> Result<(), Failed> {
    let output = v_rust_command().arg("test").arg(path).output().unwrap();

    if output.status.success() {
        return Ok(());
    }

    Err(format!(
        "v-rust test failed with status {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
    .into())
}
