use std::process::ExitCode;

use belle::{
    cli::{self, display_errors},
    config::BelleConfig,
};
use clap::Parser;

#[tokio::main]
async fn main() -> ExitCode {
    // Parse command-line arguments and dispatch to the appropriate action
    let args = cli::Cli::parse();
    let backtrace_errors = args.global_args.backtrace;

    // Ensure configuration/state is initialised
    if let Err(e) = BelleConfig::init() {
        display_errors(&e.into(), backtrace_errors);
        return ExitCode::FAILURE;
    };

    // Execute the commands
    if let Err(e) = cli::run(args).await {
        display_errors(&e, backtrace_errors);

        return ExitCode::FAILURE;
    };

    ExitCode::SUCCESS
}
