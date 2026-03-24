pub trait Hint {
    fn get_hint(&self) -> Option<Cow<'static, str>>;
}

use std::borrow::Cow;
mod hinted;

pub use hinted::{HintContainerExt, HintedResultExt};
pub use hinted_derive::Hint;

/// Wrapper storing a hints for enum
#[derive(Debug)]
pub struct Hinted<E> {
    source: E,
    hint: Option<Cow<'static, str>>,
}

impl<E: Hint> Hinted<E> {
    /// Retrieve hint for given enum
    pub fn hint(&self) -> Option<Cow<'static, str>> {
        // Priorities custom advice, then use macro set error, then none
        self.hint.clone().or_else(|| self.source.get_hint())
    }
}

impl<E> Hinted<E> {
    pub fn source(&self) -> &E {
        &self.source
    }
}
