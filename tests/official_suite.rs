use assert_cmd::Command;
use libtest_mimic::{Arguments, Trial};
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;

fn main() {
    let args = Arguments::from_args();
    let repo_dir = PathBuf::from("tests/v_official_repo");

    ensure_official_repo(&repo_dir);

    let tests = vec![Trial::test("official v test semantics", move || {
        run_official_v_tests(&repo_dir)
    })];
    libtest_mimic::run(&args, tests).exit();
}

fn ensure_official_repo(repo_dir: &Path) {
    if repo_dir.exists() {
        return;
    }

    println!("Downloading official V tests (RED GREEN REFACTOR)...");
    let status = StdCommand::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "https://github.com/vlang/v",
            repo_dir.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to clone official V repository");

    assert!(status.success(), "Failed to clone official V repository");
}

fn run_official_v_tests(repo_dir: &Path) -> Result<(), libtest_mimic::Failed> {
    let mut compiler_cmd = Command::cargo_bin("v-rust").unwrap();
    let output = compiler_cmd
        .arg("test")
        .arg(repo_dir.join("vlib"))
        .arg(repo_dir.join("cmd/v/tests"))
        .output()
        .unwrap();

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
