mod common;

use libtest_mimic::{Arguments, Trial};

fn main() {
    let args = Arguments::from_args();
    let repo_dir = common::official_repo_dir();

    common::ensure_official_repo(&repo_dir);

    let tests =
        vec![Trial::test("official v test semantics", move || common::run_v_rust_test(&repo_dir))];
    libtest_mimic::run(&args, tests).exit();
}
