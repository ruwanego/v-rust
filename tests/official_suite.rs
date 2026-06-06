mod official_common;

use libtest_mimic::{Arguments, Trial};

fn main() {
    let args = Arguments::from_args();
    let repo_dir = official_common::official_repo_dir();

    official_common::ensure_official_repo(&repo_dir);

    let tests = vec![Trial::test("official v test semantics", move || {
        official_common::run_v_rust_test(&repo_dir)
    })];
    libtest_mimic::run(&args, tests).exit();
}
