use std::error::Error;

use belle::{cli, config::BelleConfig, error::AppError};
use clap::Parser;
use console::style;

// todo format errors
fn display_errors(e: &AppError, backtrace: bool) {
    eprintln!("{} {}", style("Error:").bold().red(), style(e).bright().red());

    if backtrace {
        let mut current_source = e.source();
        let mut depth = 0;

        while let Some(source) = current_source {
            depth += 1;
            let indent = " ".repeat(depth * 2);
            let msg = source.to_string();

            // Split the error message into lines to indent them all
            for (i, line) in msg.lines().enumerate() {
                if i == 0 {
                    // Place arrow on the first line
                    eprintln!("{}⮡ {}", indent, style(line).red());
                } else {
                    eprintln!("{}   {}", indent, style(line).red());
                }
            }
            current_source = source.source();
        }
    } else if e.source().is_some() {
        eprintln!(
            "{}",
            style("help: use '--backtrace' to see error source chain").dim().italic()
        );
    }
}

#[tokio::main]
async fn main() {
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

// todo there is inconsistency between "pkg [version]" and "pkg@version"
// todo no-op commands
// todo consistent errors output
// todo error hints
// todo unit testing
// todo manual testing
