pub mod error;
mod metadata;
mod parser;
mod root_parser;
mod schema;
mod types;

use schema::{AFPAuthorMeta, AFPLicenceMeta, AFPTheoryMeta};
pub use types::RepoMetadata;
use types::{AuthorMetadata, TheoryMetadata};
