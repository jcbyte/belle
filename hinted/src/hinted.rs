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
