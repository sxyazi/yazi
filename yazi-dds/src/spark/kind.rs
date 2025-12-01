use std::fmt::Display;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SparkKind {
	// Sort
	KeySort,
	IndSort,
	// Stash
	IndStash,
	RelayStash,
	// Quit
	KeyQuit,
}

impl AsRef<str> for SparkKind {
	fn as_ref(&self) -> &str {
		match self {
			// Sort
			Self::KeySort => "key-sort",
			Self::IndSort => "ind-sort",
			// Stash
			Self::IndStash => "ind-stash",
			Self::RelayStash => "relay-stash",
			// Quit
			Self::KeyQuit => "key-quit",
		}
	}
}

impl Display for SparkKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(self.as_ref()) }
}
