use belle::cli_schema::Cli;
use belle::config::BelleConfig;
use clap::Parser;

#[tokio::main]
async fn main() {
    // Ensure configuration/state is initialised
    if let Err(e) = BelleConfig::init() {
        // todo error handling
    }

    // Parse command-line arguments and dispatch to the appropriate action
    let args = Cli::parse();

    if let Err(e) = belle::run(args).await {
        // todo error handling
    }
}
