use belle::cli;
use belle::config::BelleConfig;
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

// todo fetch theories
// todo create ROOTS files
// todo integrate with isabelle
