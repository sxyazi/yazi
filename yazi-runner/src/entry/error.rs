use strum::EnumIs;
use thiserror::Error;

#[derive(Clone, Debug, EnumIs, Error)]
pub enum EntryError {
	#[error("Entry task cancelled")]
	Cancelled,
}
