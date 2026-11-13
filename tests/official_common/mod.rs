use assert_cmd::Command;
use libtest_mimic::Failed;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;

fn v_rust_command() -> Command {
    Command::cargo_bin("v-rust").expect("v-rust binary should be built by cargo test")
}

/// Pinned full commit SHA of the official V corpus so the acceptance suite is
/// deterministic. Bump by editing `tests/v_repo_commit.txt`, then delete
/// `tests/v_official_repo` and re-run.
const V_REPO_COMMIT: &str = include_str!("../v_repo_commit.txt");

pub fn ensure_official_repo(repo_dir: &Path) {
    let pin = V_REPO_COMMIT.trim();

    if repo_dir.exists() {
        let output = StdCommand::new("git")
            .arg("-C")
            .arg(repo_dir)
            .args(["rev-parse", "HEAD"])
            .output()
            .expect("Failed to run git rev-parse in official V repository");
        let head = String::from_utf8_lossy(&output.stdout).trim().to_string();
        assert!(
            head == pin,
            "{} is at commit {head}, but tests/v_repo_commit.txt pins {pin}. \
             Delete tests/v_official_repo and re-run.",
            repo_dir.display()
        );
        return;
    }

    println!("Downloading official V tests at {pin} (RED GREEN REFACTOR)...");
    let dir = repo_dir.to_str().unwrap();
    let run = |args: &[&str]| {
        let status = StdCommand::new("git")
            .args(args)
            .status()
            .expect("Failed to run git for official V repository");
        assert!(status.success(), "git {args:?} failed");
    };
    run(&["init", "--quiet", dir]);
    run(&["-C", dir, "remote", "add", "origin", "https://github.com/vlang/v"]);
    run(&["-C", dir, "fetch", "--depth", "1", "origin", pin]);
    run(&["-C", dir, "checkout", "--quiet", "--detach", "FETCH_HEAD"]);
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
