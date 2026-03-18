use thiserror::Error;

#[derive(Error, Debug)]
pub enum CustomError {
    #[error("{msg}")]
    WithSource {
        msg: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("{msg}")]
    WithoutSource { msg: String },
}

pub trait CustomErrorContext<T> {
    fn report_custom(self, msg: impl Into<String>) -> Result<T, CustomError>;
}

impl<T> CustomErrorContext<T> for Option<T> {
    fn report_custom(self, msg: impl Into<String>) -> Result<T, CustomError> {
        self.ok_or_else(|| CustomError::WithoutSource { msg: msg.into() })
    }
}

impl<T, E> CustomErrorContext<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn report_custom(self, msg: impl Into<String>) -> Result<T, CustomError> {
        self.map_err(|e| CustomError::WithSource {
            msg: msg.into(),
            source: Box::new(e),
        })
    }
}
