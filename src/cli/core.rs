use crate::{error::AppError, registry::PackageIdentifier};
use console::{StyledObject, style};
use hinted::Hinted;
use indicatif::{ProgressBar, ProgressStyle};
use pubgrub::SemanticVersion;
use std::fmt::{self, Write};
use std::{borrow::Cow, error::Error, fmt::Display, time::Duration};

const GUTTER_WIDTH: usize = 12;

pub fn display_errors(e: &Hinted<AppError>, backtrace: bool) {
    let err = e.source();
    CliLine::new().line(err.to_string()).with_error().eprint();

    if backtrace {
        let mut current_source = err.source();
        let mut depth = 0;

        while let Some(source) = current_source {
            let indent = " ".repeat(GUTTER_WIDTH + depth * 2);
            let msg = source.to_string();

            // Split the error message into lines to indent them all
            for (i, line) in msg.lines().enumerate() {
                if i == 0 {
                    // Place arrow on the first line
                    eprintln!("{} ⮡ {}", indent, style(line).red());
                } else {
                    eprintln!("{}    {}", indent, style(line).red());
                }
            }
            current_source = source.source();
            depth += 1;
        }
    }

    // Display hint underneath error and trace
    if let Some(hint) = e.get_hint() {
        CliLine::new().line(hint).with_note().eprint();
    }

    // If a backtrace is available but not used notify
    if !backtrace && err.source().is_some() {
        CliLine::new()
            .line(format!(
                "{}{} {}",
                style("help").dim().bold(),
                style(":").dim(),
                style("use '--backtrace' to see error source chain").dim().italic()
            ))
            .eprint();
    }
}

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
        self.enable_steady_tick(Duration::from_millis(100));

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
    Note,
    Default,
}

pub struct CliLine {
    prefix: Cow<'static, str>,
    line: Cow<'static, str>,
    intent: CliLineIntent,
    custom_prefix: bool,
}

impl CliLine {
    pub fn new() -> Self {
        Self {
            prefix: Cow::<str>::default(),
            line: Cow::<str>::default(),
            intent: CliLineIntent::Default,
            custom_prefix: false,
        }
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
            CliLineIntent::Note => styled_prefix,
            CliLineIntent::Default => styled_prefix,
        }
    }

    pub fn style_line<'a>(line: &'a str, intent: CliLineIntent) -> Cow<'a, str> {
        match intent {
            CliLineIntent::Success => Cow::Borrowed(line),
            CliLineIntent::Focus => Cow::Borrowed(line),
            CliLineIntent::Warning => Cow::Owned(style(console::strip_ansi_codes(line)).yellow().to_string()),
            CliLineIntent::Error => Cow::Owned(style(console::strip_ansi_codes(line)).red().bright().to_string()),
            CliLineIntent::Skipped => Cow::Borrowed(line),
            CliLineIntent::Note => Cow::Owned(format!("{}: {}", style("note").bold().bright(), line)),
            CliLineIntent::Default => Cow::Borrowed(line),
        }
    }

    pub fn get(&self) -> String {
        let prefix_value = if !self.custom_prefix {
            // Replace prefix with hardcoded values for certain intents
            match self.intent {
                CliLineIntent::Error => "Error",
                CliLineIntent::Warning => "Warning",
                CliLineIntent::Skipped => "Skipped",
                CliLineIntent::Note => "",
                _ => &self.prefix,
            }
        } else {
            // Do not replaced custom prefixes
            &self.prefix
        };

        // Ensure gutter space on styled prefix
        let padded_prefix = format!("{:>width$}", prefix_value, width = GUTTER_WIDTH);
        let mut result = format!("{} ", Self::style_prefix(padded_prefix, self.intent),);

        // Style line, add it line-by-line including padding if it is multi-line
        let styled_line = Self::style_line(&self.line, self.intent);
        for (i, l) in styled_line.lines().enumerate() {
            if i == 0 {
                // Do not add padding to the first line as this follows directly from the prefix
                writeln!(result, "{}", l).expect("Writing to a String failed");
            } else {
                writeln!(result, "{:width$} {}", "", l, width = GUTTER_WIDTH).expect("Writing to a String failed");
            }
        }

        result
    }

    pub fn print(&self) {
        print!("{}", self.get());
    }

    pub fn eprint(&self) {
        eprint!("{}", self.get());
    }

    pub fn prefix(mut self, prefix: impl Into<Cow<'static, str>>) -> Self {
        self.prefix = prefix.into();
        self.custom_prefix = true;
        self
    }

    pub fn line(mut self, line: impl Into<Cow<'static, str>>) -> Self {
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

    pub fn with_note(mut self) -> Self {
        self.intent = CliLineIntent::Note;
        self
    }
}

pub enum DisplayVersion<'a> {
    /// User explicitly defined version
    Explicit(&'a SemanticVersion),
    /// System inferred though resolving
    Implicit(&'a SemanticVersion),
}

impl<'a> fmt::Display for DisplayVersion<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DisplayVersion::Explicit(v) => {
                let s = format!("v{}", v);
                f.pad(&s)
            }
            DisplayVersion::Implicit(v) => {
                let s = format!("v{}", v);

                if let Some(width) = f.width() {
                    write!(f, "{:width$}", style(s).dim(), width = width)
                } else {
                    write!(f, "{}", style(s).dim())
                }
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pluralise() {
        assert_eq!(pluralise(1, "apple", "apples"), "apple");
        assert_eq!(pluralise(2, "apple", "apples"), "apples");
        assert_eq!(pluralise(0, "apple", "apples"), "apples");
    }

    #[test]
    fn test_cli_line_multi_line() {
        let line = CliLine::new().prefix("Test").line("Line 1\nLine 2").get();

        let lines: Vec<&str> = line.lines().collect();
        // Check if second line is indented by GUTTER_WIDTH (12)
        assert!(lines[1].starts_with("            "));
    }

    #[test]
    fn test_cli_line_intent() {
        let err_line = CliLine::new().with_error().get();
        assert!(err_line.contains("Error"));

        let warning_line = CliLine::new().with_warning().get();
        assert!(warning_line.contains("Warning"));

        let skipped_line = CliLine::new().with_skipped().get();
        assert!(skipped_line.contains("Skipped"));

        let note_line = CliLine::new().with_note().get();
        assert!(note_line.contains("note"));
    }

    #[test]
    fn test_cli_line_custom_intent() {
        let err_line = CliLine::new().prefix("Test").with_error().get();
        assert!(!err_line.contains("Error"));

        let warning_line = CliLine::new().prefix("Test").with_warning().get();
        assert!(!warning_line.contains("Warning"));

        let skipped_line = CliLine::new().prefix("Test").with_skipped().get();
        assert!(!skipped_line.contains("Skipped"));
    }
}
