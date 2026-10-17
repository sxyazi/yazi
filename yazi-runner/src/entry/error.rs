use strum::EnumIs;
use thiserror::Error;

#[derive(Clone, Debug, EnumIs, Error)]
pub enum EntryError {
	#[error("Entry task cancelled")]
	Cancelled,
}

impl From<EntryError> for mlua::Error {
	fn from(value: EntryError) -> Self { Self::external(value) }
}
