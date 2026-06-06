use assert_cmd::Command;

pub fn v_rust_command() -> Command {
    Command::cargo_bin("v-rust").expect("v-rust binary should be built by cargo test")
}
