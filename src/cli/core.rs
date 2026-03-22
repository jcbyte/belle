use indicatif::{ProgressBar, ProgressStyle};
use pubgrub::SemanticVersion;
use std::fmt::Display;

use console::{Color, style};

pub trait ProgressBarTheme {
    fn with_belle_style(self) -> Self;
}

impl ProgressBarTheme for ProgressBar {
    fn with_belle_style(self) -> Self {
        self.set_style(
            ProgressStyle::default_bar()
                .template("{spinner} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .expect("Invalid hardcoded spinner template")
                .progress_chars("#>-"),
        );
        self
    }
}

const GUTTER_WIDTH: usize = 12;

pub fn print_ln<T: Display>(prefix: &str, color: console::Color, line: T) {
    println!(
        "{:>width$} {}",
        style(prefix).fg(color).bold(),
        line,
        width = GUTTER_WIDTH
    );
}

pub fn print_blank_ln<T: Display>(line: T) {
    println!("{:width$} {}", "", line, width = GUTTER_WIDTH);
}

pub fn print_success_ln<T: Display>(prefix: &str, line: T) {
    print_ln(prefix, Color::Green, line);
}

pub fn print_warning_ln<T: Display>(line: T) {
    print_ln("Warning", Color::Yellow, style(line).yellow());
}

pub fn print_skipped_ln<T: Display>(line: T) {
    print_ln("Skipped", Color::White, style(line).dim());
}

pub enum DisplayVersion<'a> {
    /// User explicitly defined version
    Explicit(&'a SemanticVersion),
    /// System inferred though resolving
    Implicit(&'a SemanticVersion),
}

use std::fmt;

impl<'a> fmt::Display for DisplayVersion<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DisplayVersion::Explicit(v) => write!(f, "[{}]", v),
            DisplayVersion::Implicit(v) => write!(f, "{}", style(format!("[{}]", v)).dim()),
        }
    }
}

impl<'a> DisplayVersion<'a> {
    pub fn get_version(&self) -> &'a SemanticVersion {
        match self {
            DisplayVersion::Explicit(v) => v,
            DisplayVersion::Implicit(v) => v,
        }
    }
}
