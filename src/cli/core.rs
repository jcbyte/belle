use indicatif::{ProgressBar, ProgressStyle};
use pubgrub::SemanticVersion;
use std::{borrow::Cow, fmt::Display};

use console::{StyledObject, style};

const GUTTER_WIDTH: usize = 12;

pub trait ProgressBarTheme {
    fn with_belle_bar_style(self) -> Self;
    fn with_belle_spinner_style(self) -> Self;
    fn set_belle_prefix<P: Display>(&self, prefix: P);
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

    fn set_belle_prefix<P: Display>(&self, prefix: P) {
        self.set_prefix(CliLine::style_prefix(prefix, CliLineIntent::Focus).to_string());
    }
}

#[derive(Copy, Clone)]
pub enum CliLineIntent {
    Success,
    Focus,
    Warning,
    Error,
    Skipped,
    Default,
}

pub struct CliLine {
    prefix: String,
    line: String,
    intent: CliLineIntent,
    custom_prefix: bool,
}

impl CliLine {
    pub fn new() -> Self {
        Self {
            prefix: String::new(),
            line: String::new(),
            intent: CliLineIntent::Default,
            custom_prefix: false,
        }
    }

    pub fn get(&self) -> String {
        let prefix_value = if !self.custom_prefix {
            // Replace prefix with hardcoded values for certain intents
            match self.intent {
                CliLineIntent::Error => "Error",
                CliLineIntent::Warning => "Warning",
                CliLineIntent::Skipped => "Skipped",
                _ => &self.prefix,
            }
        } else {
            // Do not replaced custom prefixes
            &self.prefix
        };

        // Ensure gutter space
        let padded_prefix = format!("{:>width$}", prefix_value, width = GUTTER_WIDTH);

        // Format the prefix and line according to intent
        format!(
            "{} {}",
            Self::style_prefix(padded_prefix, self.intent),
            Self::style_line(&self.line, self.intent)
        )
    }

    pub fn style_prefix<P>(prefix: P, intent: CliLineIntent) -> StyledObject<P>
    where
        P: Display,
    {
        let styled_prefix = style(prefix).bold();

        match intent {
            CliLineIntent::Success => styled_prefix.green(),
            CliLineIntent::Focus => styled_prefix.cyan(),
            CliLineIntent::Warning => styled_prefix.yellow(),
            CliLineIntent::Error => styled_prefix.red(),
            CliLineIntent::Skipped => styled_prefix.dim(),
            CliLineIntent::Default => styled_prefix,
        }
    }

    pub fn style_line<'a>(line: &'a str, intent: CliLineIntent) -> Cow<'a, str> {
        match intent {
            CliLineIntent::Success => Cow::Borrowed(line),
            CliLineIntent::Focus => Cow::Borrowed(line),
            CliLineIntent::Warning => Cow::Owned(style(console::strip_ansi_codes(line)).yellow().to_string()),
            CliLineIntent::Error => Cow::Owned(style(console::strip_ansi_codes(line)).red().to_string()),
            CliLineIntent::Skipped => Cow::Borrowed(line),
            CliLineIntent::Default => Cow::Borrowed(line),
        }
    }

    pub fn print(&self) {
        println!("{}", self.get());
    }

    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self.custom_prefix = true;
        self
    }

    pub fn line(mut self, line: impl Into<String>) -> Self {
        self.line = line.into();
        self
    }

    pub fn with_success(mut self) -> Self {
        self.intent = CliLineIntent::Success;

        self
    }

    pub fn with_focus(mut self) -> Self {
        self.intent = CliLineIntent::Focus;

        self
    }

    pub fn with_error(mut self) -> Self {
        self.intent = CliLineIntent::Error;

        self
    }

    pub fn with_warning(mut self) -> Self {
        self.intent = CliLineIntent::Warning;
        self
    }

    pub fn with_skipped(mut self) -> Self {
        self.intent = CliLineIntent::Skipped;
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
