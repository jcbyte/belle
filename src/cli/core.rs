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

pub struct CliLine {
    prefix: String,
    line: String,
}

impl CliLine {
    pub fn new() -> Self {
        Self {
            prefix: String::new(),
            line: String::new(),
        }
    }

    pub fn get(&self) -> String {
        format!("{:>width$} {}", self.prefix, self.line, width = GUTTER_WIDTH)
    }

    pub fn print(&self) {
        println!("{}", self.get());
    }

    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    pub fn line(mut self, line: impl Into<String>) -> Self {
        self.line = line.into();
        self
    }

    pub fn style_prefix<T: Display>(prefix: T, color: console::Color) -> StyledObject<T> {
        style(prefix).fg(color).bold()
    }

    pub fn style_success_prefix<T: Display>(prefix: T) -> StyledObject<T> {
        Self::style_prefix(prefix, Color::Green)
    }

    pub fn style_focus_prefix<T: Display>(prefix: T) -> StyledObject<T> {
        Self::style_prefix(prefix, Color::Cyan)
    }

    pub fn as_success(mut self) -> Self {
        self.prefix = Self::style_success_prefix(self.prefix).to_string();
        self
    }

    pub fn as_focus(mut self) -> Self {
        self.prefix = Self::style_focus_prefix(self.prefix).to_string();
        self
    }

    pub fn as_error(mut self) -> Self {
        self.prefix = Self::style_prefix(self.prefix, Color::Red).to_string();
        self.line = style(self.line).red().to_string();
        self
    }

    pub fn as_warning(mut self) -> Self {
        self.prefix = Self::style_prefix(self.prefix, Color::Yellow).to_string();
        self.line = style(self.line).yellow().to_string();
        self
    }
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
