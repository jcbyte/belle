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
        for cause in e.chain() {
            println!("- {}", style(cause).bold().red())
        }
    }
}

// todo save packages as PackageIdentifiers+Isabelles with custom serialiser

// todo 6.1 ensure consistent naming of packages
// todo 6.2 check all error handling cases are needed (should we just expect), ensure messages are correct (resolving, deserialising etc), use thiserror
// todo 6.3 use references instead of cloning everywhere
// todo 6.4 consistent CLI output

// todo 7 unix-ify junctions/symlinks + isabelle integrations

// todo 8 unit testing
// todo 9 integration testing
