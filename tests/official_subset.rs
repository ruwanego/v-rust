mod official_common;

use libtest_mimic::{Arguments, Failed, Trial};
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let args = Arguments::from_args();
    let manifest = PathBuf::from("tests/official_subset.txt");
    let repo_dir = official_common::official_repo_dir();
    let subset = read_subset_manifest(&manifest);

    if subset.is_empty() {
        let tests = vec![Trial::test("official subset manifest is empty", || Ok(()))];
        libtest_mimic::run(&args, tests).exit();
    }

    official_common::ensure_official_repo(&repo_dir);

    let tests = subset
        .into_iter()
        .map(|relative_path| {
            let test_path = repo_dir.join(&relative_path);
            let name = format!("official subset::{}", relative_path.display());
            Trial::test(name, move || run_subset_case(&test_path))
        })
        .collect();

    libtest_mimic::run(&args, tests).exit();
}

fn read_subset_manifest(manifest: &Path) -> Vec<PathBuf> {
    let contents = fs::read_to_string(manifest)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", manifest.display()));

    contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(PathBuf::from)
        .collect()
}

fn run_subset_case(test_path: &Path) -> Result<(), Failed> {
    if !test_path.exists() {
        return Err(format!("official subset path does not exist: {}", test_path.display()).into());
    }

    official_common::run_v_rust_test(test_path)
}
