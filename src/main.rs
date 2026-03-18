use std::error::Error;

use belle::{cli, config::BelleConfig, error::AppError};
use clap::Parser;
use console::style;

fn display_errors(e: &AppError) {
    println!("{}", style(e).bold().red());
    if e.source().is_some() {
        // todo is this a good idea
        // todo error hints
        println!("{}", style("use '--source' to see original source").dim())
    }
}

#[tokio::main]
async fn main() {
    // Ensure configuration/state is initialised
    if let Err(e) = BelleConfig::init() {
        display_errors(&e);
        return;
    }

    // Parse command-line arguments and dispatch to the appropriate action
    let args = cli::Cli::parse();

    // Execute the commands
    if let Err(e) = cli::run(args).await {
        display_errors(&e);
        return;
    }
}

// todo 7.3 use references instead of cloning everywhere
// todo 7.1 ensure consistent naming of packages
// todo 7.4 consistent CLI output

// todo readme

// todo 8 unit testing
// todo 9 integration testing
