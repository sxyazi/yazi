use thiserror::Error;

#[derive(Debug, Error)]
#[error("invalid locale date format")]
pub struct LocaleError;

impl From<&LocaleError> for LocaleError {
	fn from(_: &LocaleError) -> Self { Self }
}

impl From<std::fmt::Error> for LocaleError {
	fn from(_: std::fmt::Error) -> Self { Self }
}

impl From<LocaleError> for mlua::Error {
	fn from(err: LocaleError) -> Self { Self::runtime(err) }
}
