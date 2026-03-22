use indicatif::{ProgressBar, ProgressStyle};
use pubgrub::SemanticVersion;
use std::fmt::Display;

use console::{Color, StyledObject, style};

const GUTTER_WIDTH: usize = 12;

pub trait ProgressBarTheme {
    fn with_belle_bar_style(self) -> Self;
    fn with_belle_spinner_style(self) -> Self;
}

impl ProgressBarTheme for ProgressBar {
    fn with_belle_bar_style(self) -> Self {
        self.set_style(
            ProgressStyle::default_bar()
                .template(&format!(
                    "{{prefix:>{}}} [{{bar:40.cyan/blue}}] {{pos}}/{{len}}: {{msg}}",
                    GUTTER_WIDTH
                ))
                .expect("Invalid hardcoded progressbar template")
                .progress_chars("=> "),
        );
        self
    }

    fn with_belle_spinner_style(self) -> Self {
        self.set_style(
            ProgressStyle::default_spinner()
                .template(&format!(
                    "{{spinner}} {{prefix:>{}}} {{msg}}",
                    // Giving space for: spinner + space
                    GUTTER_WIDTH - 2
                ))
                .expect("Invalid hardcoded spinner template"),
        );

        self
    }
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

use crate::registry::PackageIdentifier;

impl<'a> fmt::Display for DisplayVersion<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DisplayVersion::Explicit(v) => write!(f, "v{}", v),
            DisplayVersion::Implicit(v) => write!(f, "{}", style(format!("v{}", v)).dim()),
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

impl PackageIdentifier {
    pub fn styled(&self) -> String {
        format!(
            "{} {}",
            style(&self.name).cyan().bright(),
            DisplayVersion::Implicit(&self.version)
        )
    }
}

pub fn pluralise<'a>(count: usize, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 { singular } else { plural }
}
