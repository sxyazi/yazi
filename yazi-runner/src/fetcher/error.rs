use strum::EnumIs;
use thiserror::Error;

#[derive(Clone, Debug, EnumIs, Error)]
pub enum FetchError {
	#[error("Fetch task cancelled")]
	Cancelled,
}
