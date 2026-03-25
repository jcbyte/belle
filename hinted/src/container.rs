use std::borrow::Cow;

use crate::Hinted;

// General lifting conversion
impl<E> From<E> for Hinted<E> {
    fn from(source: E) -> Self {
        Self { source, hint: None }
    }
}

pub trait HintedResultExt<T, E> {
    /// Lift an error type into a Hinted type
    fn into_hinted<BigErr>(self) -> Result<T, Hinted<BigErr>>
    where
        BigErr: From<E>;

    /// Add a custom hint to a raw error, wrapping it in Hinted.
    fn hint<BigErr>(self, hint: impl Into<Cow<'static, str>>) -> Result<T, Hinted<BigErr>>
    where
        BigErr: From<E>;
}

impl<T, E> HintedResultExt<T, E> for Result<T, E> {
    fn into_hinted<BigErr>(self) -> Result<T, Hinted<BigErr>>
    where
        BigErr: From<E>,
    {
        self.map_err(|e| Hinted {
            source: e.into(),
            hint: None,
        })
    }

    fn hint<BigErr>(self, hint: impl Into<Cow<'static, str>>) -> Result<T, Hinted<BigErr>>
    where
        BigErr: From<E>,
    {
        self.map_err(|e| Hinted {
            source: e.into(),
            hint: Some(hint.into()),
        })
    }
}

pub trait HintContainerExt<T, E> {
    /// Convert an existing Hinted to a larger error type
    fn map_hinted<BigErr>(self) -> Result<T, Hinted<BigErr>>
    where
        BigErr: From<E>;

    /// Add a custom hint to a Hinted, replacing the current hint
    fn rehint<BigErr>(self, hint: impl Into<Cow<'static, str>>) -> Result<T, Hinted<BigErr>>
    where
        BigErr: From<E>;
}

impl<T, E> HintContainerExt<T, E> for Result<T, Hinted<E>> {
    fn map_hinted<BigErr>(self) -> Result<T, Hinted<BigErr>>
    where
        BigErr: From<E>,
    {
        self.map_err(|hinted| Hinted {
            source: hinted.source.into(),
            hint: hinted.hint,
        })
    }

    fn rehint<BigErr>(self, hint: impl Into<Cow<'static, str>>) -> Result<T, Hinted<BigErr>>
    where
        BigErr: From<E>,
    {
        self.map_err(|existing| Hinted {
            source: existing.source.into(),
            hint: Some(hint.into()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A simple error type for testing
    #[derive(Debug, PartialEq)]
    struct InnerError;

    // A "larger" error type that can wrap InnerError
    #[derive(Debug, PartialEq)]
    enum OuterError {
        Wrapped(InnerError),
    }

    impl From<InnerError> for OuterError {
        fn from(e: InnerError) -> Self {
            OuterError::Wrapped(e)
        }
    }

    #[test]
    fn test_into_hinted_lifts_error() {
        let res: Result<(), InnerError> = Err(InnerError);
        let hinted_res: Result<(), Hinted<OuterError>> = res.into_hinted();

        let err = hinted_res.unwrap_err();
        assert_eq!(err.source, OuterError::Wrapped(InnerError));
        assert_eq!(err.hint, None);
    }

    #[test]
    fn test_hint_adds_metadata() {
        let res: Result<(), InnerError> = Err(InnerError);
        let hinted_res: Result<(), Hinted<InnerError>> = res.hint("Try turning it off and on");

        let err = hinted_res.unwrap_err();
        assert_eq!(err.source, InnerError);
        assert!(err.hint.is_some());
        assert_eq!(err.hint.unwrap(), "Try turning it off and on");
    }

    #[test]
    fn test_map_hinted_converts_error_type() {
        let initial: Result<(), Hinted<InnerError>> = Err(Hinted {
            source: InnerError,
            hint: Some("Initial hint".into()),
        });

        let mapped: Result<(), Hinted<OuterError>> = initial.map_hinted();

        let err = mapped.unwrap_err();
        assert_eq!(err.source, OuterError::Wrapped(InnerError));
        assert!(err.hint.is_some());
        assert_eq!(err.hint.unwrap(), "Initial hint");
    }

    #[test]
    fn test_rehint_overwrites_existing_hint() {
        let initial: Result<(), Hinted<InnerError>> = Err(Hinted {
            source: InnerError,
            hint: Some("Original hint".into()),
        });

        let rehinted: Result<(), Hinted<InnerError>> = initial.rehint("New hint");
        let err = rehinted.unwrap_err();
        assert_eq!(err.source, InnerError);
        assert!(err.hint.is_some());
        assert_eq!(err.hint.unwrap(), "New hint");
    }
}
