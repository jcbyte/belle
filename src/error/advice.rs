use std::borrow::Cow;

use strum::EnumMessage;

/// Wrapper type to store a hint for errors
pub struct Advice<E> {
    pub source: E,
    pub advice: Option<Cow<'static, str>>,
}

impl<E: EnumMessage> Advice<E> {
    /// Retrieve advice for given error
    pub fn advice(&self) -> Option<Cow<'static, str>> {
        // Priorities custom advice, then use strum set error, then none
        self.advice.clone().or_else(|| self.source.get_message().map(Cow::Borrowed))
    }
}

// General lifting conversion
impl<E> From<E> for Advice<E> {
    fn from(source: E) -> Self {
        Self { source, advice: None }
    }
}

pub trait AdviceResultExt<T, E> {
    /// Lift an error type into Advice
    fn into_advice<BigErr>(self) -> Result<T, Advice<BigErr>>
    where
        BigErr: From<E>;

    /// Add a custom advice to a raw error, wrapping it in Advice.
    fn advise<BigErr>(self, advice: impl Into<Cow<'static, str>>) -> Result<T, Advice<BigErr>>
    where
        BigErr: From<E>;
}

impl<T, E> AdviceResultExt<T, E> for Result<T, E> {
    fn into_advice<BigErr>(self) -> Result<T, Advice<BigErr>>
    where
        BigErr: From<E>,
    {
        self.map_err(|e| Advice {
            source: e.into(),
            advice: None,
        })
    }

    fn advise<BigErr>(self, advice: impl Into<Cow<'static, str>>) -> Result<T, Advice<BigErr>>
    where
        BigErr: From<E>,
    {
        self.map_err(|e| Advice {
            source: e.into(),
            advice: Some(advice.into()),
        })
    }
}

pub trait AdviceContainerExt<T, E> {
    /// Convert an existing Advice to a larger error type
    fn map_advice<BigErr>(self) -> Result<T, Advice<BigErr>>
    where
        BigErr: From<E>;

    /// Add a custom advice to a wrapped error, replacing the current advice
    fn readvise<BigErr>(self, advice: impl Into<Cow<'static, str>>) -> Result<T, Advice<BigErr>>
    where
        BigErr: From<E>;
}

impl<T, E> AdviceContainerExt<T, E> for Result<T, Advice<E>> {
    fn map_advice<BigErr>(self) -> Result<T, Advice<BigErr>>
    where
        BigErr: From<E>,
    {
        self.map_err(|a| Advice {
            source: a.source.into(),
            advice: a.advice,
        })
    }

    fn readvise<BigErr>(self, advice: impl Into<Cow<'static, str>>) -> Result<T, Advice<BigErr>>
    where
        BigErr: From<E>,
    {
        self.map_err(|a| Advice {
            source: a.source.into(),
            advice: Some(advice.into()),
        })
    }
}
