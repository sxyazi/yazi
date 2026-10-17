use strum::EnumIs;
use thiserror::Error;

#[derive(Clone, Debug, EnumIs, Error)]
pub enum EvaluateError {
	#[error("Async blocking task cancelled")]
	Cancelled,
}
