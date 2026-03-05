mod error_wrapper;
mod resolver;

use error_wrapper::SolverError;
pub use resolver::BelleDependencyProvider;

pub static ISABELLE_PACKAGE: &str = "!Isabelle";
