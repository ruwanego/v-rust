#![forbid(unsafe_code)]

mod driver;

fn main() -> std::process::ExitCode {
    driver::cli::run()
}
