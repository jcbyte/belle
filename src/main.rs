use belle::{cli, config::BelleConfig};
use clap::Parser;
use console::style;

#[tokio::main]
async fn main() {
    // Ensure configuration/state is initialised
    if let Err(e) = BelleConfig::init() {
        // todo error handling
    }

    // Parse command-line arguments and dispatch to the appropriate action
    let args = cli::Cli::parse();

    // Execute the commands
    if let Err(e) = cli::run(args).await {
        // todo error handling
        println!("{}", style(e).bold().red())
    }
}

// todo 2 there should be a way to set environment isabelle version

// todo fetch theories
// todo create ROOTS files
// todo integrate with isabelle

// todo CI
// todo ensure consistent naming of packages
// todo check all error handling cases are needed (should we just expect), ensure messages are correct (resolving, deserialising etc)
// todo testing
