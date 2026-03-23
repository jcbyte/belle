use hinted::Hint;
use thiserror::Error;

#[derive(Error, Debug, Hint)]
pub enum CliError {
    #[error("no environment is currently active")]
    #[hint("use `belle switch <name>` to select one, or `belle env create <name>` to create one")]
    NoActiveEnvironment,
}
