use belle::{cli, config::BelleConfig};
use clap::Parser;
use console::style;

#[tokio::main]
async fn main() {
    // Ensure configuration/state is initialised
    if let Err(e) = BelleConfig::init() {
        // todo 6.2 error handling
    }

    // Parse command-line arguments and dispatch to the appropriate action
    let args = cli::Cli::parse();

    // Execute the commands
    if let Err(e) = cli::run(args).await {
        // todo 6.2 error handling
        println!("{}", style(e).bold().red())
    }
}

// todo 3 fetch theories

// todo 4 create ROOTS files
// todo 4 integrate with isabelle

// todo 6.1 ensure consistent naming of packages
// todo 6.2 check all error handling cases are needed (should we just expect), ensure messages are correct (resolving, deserialising etc), use thiserror
// todo 6.3 use references instead of cloning everywhere

// todo 7 testing
// todo 7 CI
