mod core;
mod error_wrapper;

pub use core::BelleDependencyProvider;
use error_wrapper::SolverError;

pub static ISABELLE_PACKAGE: &str = "!Isabelle";
