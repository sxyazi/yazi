use strum::EnumIs;
use thiserror::Error;

#[derive(Clone, Debug, EnumIs, Error)]
pub enum SpotError {
	#[error("Spot task cancelled")]
	Cancelled,
}
