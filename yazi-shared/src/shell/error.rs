use thiserror::Error;

#[derive(Debug, Error)]
pub enum SplitError {
	#[error("missing closing single quote")]
	MissingSingleQuote,
	#[error("missing closing double quote")]
	MissingDoubleQuote,
	#[error("missing quote after escape slash")]
	MissingQuoteAfterSlash,
}
