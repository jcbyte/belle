mod afp_schema;
mod dependency;
mod metadata;
mod parser;
mod types;

use afp_schema::{AFPAuthorEmailMeta, AFPAuthorMeta, AFPLicenceMeta, AFPTheoryMeta, AFPTheoryRelatedMeta};
pub use types::RepoMetadata;
use types::{AuthorMetadata, TheoryMetadata};
