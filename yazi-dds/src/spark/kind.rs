use std::fmt::Display;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SparkKind {
	IndStash,
	KeyQuit,
	RelayStash,
}

impl AsRef<str> for SparkKind {
	fn as_ref(&self) -> &str {
		match self {
			Self::IndStash => "ind-stash",
			Self::KeyQuit => "key-quit",
			Self::RelayStash => "relay-stash",
		}
	}
}

impl Display for SparkKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(self.as_ref()) }
}
