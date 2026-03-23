use belle::{
    cli::{self, display_errors},
    config::BelleConfig,
};
use clap::Parser;
use console::style;

#[tokio::main]
async fn main() {
    // todo remove this, this is for CLI prettyness debugging
    println!(
        "\n\n{} {} {} {} some command --here",
        style("C:/Users/joelc/Desktop/belle").bright(),
        style("main").true_color(241, 184, 12),
        style("").bold(),
        style("belle").yellow()
    );

    // Parse command-line arguments and dispatch to the appropriate action
    let args = cli::Cli::parse();
    let backtrace_errors = args.global_args.backtrace;

    // Ensure configuration/state is initialised
    if let Err(e) = BelleConfig::init() {
        display_errors(&e, backtrace_errors);
        return;
    }

    // Execute the commands
    if let Err(e) = cli::run(args).await {
        display_errors(&e, backtrace_errors);
        return;
    }
}

// todo consistent errors output
// todo error hints
// todo unit testing
// todo manual testing
