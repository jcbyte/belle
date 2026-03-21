use indicatif::{ProgressBar, ProgressStyle};
use pubgrub::SemanticVersion;
use std::fmt::Display;

use console::style;

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
    print_ln(prefix, console::Color::Green, line);
}

pub fn print_warning_ln<T: Display>(line: T) {
    print_ln("Warning", console::Color::Yellow, style(line).yellow());
}

pub enum DisplayVersion<'a> {
    /// User explicitly defined version
    Pinned(&'a SemanticVersion),
    /// System inferred though resolving
    Resolved(&'a SemanticVersion),
}

use std::fmt;

impl<'a> fmt::Display for DisplayVersion<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DisplayVersion::Pinned(v) => write!(f, "[{}]", v),
            DisplayVersion::Resolved(v) => write!(f, "{}", style(format!("[*{}]", v)).dim()),
        }
    }
}

impl<'a> DisplayVersion<'a> {
    pub fn get_version(&self) -> &'a SemanticVersion {
        match self {
            DisplayVersion::Pinned(v) => v,
            DisplayVersion::Resolved(v) => v,
        }
    }
}
