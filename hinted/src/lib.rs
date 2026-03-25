extern crate self as hinted;

pub trait Hint {
    fn get_hint(&self) -> Option<Cow<'static, str>>;
}

use std::borrow::Cow;
mod container;

pub use container::{HintContainerExt, HintedResultExt};
pub use hinted_derive::Hint;

/// Wrapper storing a hints for enum
#[derive(Debug)]
pub struct Hinted<E> {
    source: E,
    hint: Option<Cow<'static, str>>,
}

impl<E: Hint> Hinted<E> {
    /// Retrieve hint for given enum
    pub fn get_hint(&self) -> Option<Cow<'static, str>> {
        // Priorities custom advice, then use macro set error, then none
        self.hint.clone().or_else(|| self.source.get_hint())
    }
}

impl<E> Hinted<E> {
    pub fn source(&self) -> &E {
        &self.source
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hinted_derive::Hint;

    #[derive(Debug, PartialEq, Hint)]
    enum InnerError {
        #[hint("Check your internet")]
        NoConnection,
    }

    #[derive(Debug, PartialEq, Hint)]
    enum AppError {
        #[hint("Generic error")]
        Generic,

        #[hint("File not found: {arg0}")]
        NotFound(String),

        #[hint("User {id} has no permission")]
        PermissionDenied {
            id: u32,
        },

        #[hint(transparent)]
        Network(InnerError),

        // No hint attribute
        Silent,
    }

    #[test]
    fn test_macro_unit() {
        let err = AppError::Generic;
        let hint = err.get_hint();
        assert!(hint.is_some());
        assert_eq!(hint.unwrap(), "Generic error");
    }

    #[test]
    fn test_macro_unnamed_fields() {
        let err = AppError::NotFound("config.toml".to_string());
        let hint = err.get_hint();
        assert!(hint.is_some());
        assert_eq!(hint.unwrap(), "File not found: config.toml".to_string());
    }

    #[test]
    fn test_macro_named_fields() {
        let err = AppError::PermissionDenied { id: 42 };
        let hint = err.get_hint();
        assert!(hint.is_some());
        assert_eq!(hint.unwrap(), "User 42 has no permission");
    }

    #[test]
    fn test_macro_transparent() {
        let err = AppError::Network(InnerError::NoConnection);
        let hint = err.get_hint();
        assert!(hint.is_some());
        assert_eq!(hint.unwrap(), "Check your internet");
    }

    #[test]
    fn test_macro_no_attribute() {
        let err = AppError::Silent;
        assert!(err.get_hint().is_none());
    }

    #[test]
    fn test_hinted_priority_logic() {
        // Manual hint overrides macro hint
        let hinted = Hinted {
            source: AppError::Generic,
            hint: Some(Cow::Borrowed("Manual Override")),
        };
        assert_eq!(hinted.get_hint().unwrap(), "Manual Override");

        // Fallback to macro hint if manual is None
        let hinted_fallback = Hinted {
            source: AppError::Generic,
            hint: None,
        };
        assert_eq!(hinted_fallback.get_hint().unwrap(), "Generic error");

        let no_hint = Hinted {
            source: AppError::Silent,
            hint: None,
        };
        assert_eq!(no_hint.get_hint(), None);
    }
}
