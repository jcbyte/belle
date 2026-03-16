use anyhow::Error;
use belle::{cli, config::BelleConfig};
use clap::Parser;
use console::style;

fn display_errors(e: Error) {
    // todo 7.2 error handling
    for cause in e.chain() {
        println!("- {}", style(cause).bold().red())
    }
}

#[tokio::main]
async fn main() {
    // Ensure configuration/state is initialised
    if let Err(e) = BelleConfig::init() {
        display_errors(e);
        return;
    }

    // Parse command-line arguments and dispatch to the appropriate action
    let args = cli::Cli::parse();

    // Execute the commands
    if let Err(e) = cli::run(args).await {
        display_errors(e);
        return;
    }
}

// todo 7.1 ensure consistent naming of packages
// todo 7.2 check all error handling cases are needed (should we just expect), ensure messages are correct (resolving, deserialising etc), use thiserror
// todo 7.3 use references instead of cloning everywhere
// todo 7.4 consistent CLI output

// todo readme

// todo 8 unit testing
// todo 9 integration testing
