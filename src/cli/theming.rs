use indicatif::{ProgressBar, ProgressStyle};

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
