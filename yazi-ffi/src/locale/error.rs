use thiserror::Error;

#[derive(Debug, Error)]
#[error("invalid locale date format")]
pub struct LocaleError;
