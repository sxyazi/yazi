use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum Source {
	#[default]
	Unknown,

	Key,
	Ind,

	Emit,
	EmitInd,

	Relay,
	RelayInd,
}

impl Source {
	#[inline]
	pub fn is_key(self) -> bool { self == Self::Key }
}
