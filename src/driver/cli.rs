use super::test_runner;
use clap::{Args, CommandFactory, Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;
use v_rust::compiler;

#[derive(Parser, Debug)]
#[command(author, version, about = "V Compiler written in Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// The .v file to compile
    input: Option<PathBuf>,

    /// Output binary name
    #[arg(short, long, default_value = "a.out")]
    output: PathBuf,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Recursively run V _test.v files.
    Test(TestArgs),
}

#[derive(Args, Debug)]
struct TestArgs {
    /// Files or directories to test.
    paths: Vec<PathBuf>,
}

pub(crate) fn run() -> ExitCode {
    let cli = Cli::parse();

    match execute(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(CliError::Usage) => {
            let mut command = Cli::command();
            command.print_help().unwrap_or_else(|e| eprintln!("{e}"));
            eprintln!();
            ExitCode::from(2)
        }
        Err(CliError::Failure(err)) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}

fn execute(cli: &Cli) -> Result<(), CliError> {
    match &cli.command {
        Some(Command::Test(args)) => test_runner::run(&args.paths).map_err(CliError::Failure),
        None => {
            let input = cli.input.as_ref().ok_or(CliError::Usage)?;
            compiler::compile_file(input, &cli.output).map_err(CliError::Failure)
        }
    }
}

#[derive(Debug)]
enum CliError {
    Failure(String),
    Usage,
}
