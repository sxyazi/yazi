use std::fmt::Display;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SparkKind {
	// sort
	KeySort,
	IndSort,
	// stash
	IndStash,
	RelayStash,
	// quit
	KeyQuit,
	// which:show
	IndWhichShow,
}

impl AsRef<str> for SparkKind {
	fn as_ref(&self) -> &str {
		match self {
			// sort
			Self::KeySort => "key-sort",
			Self::IndSort => "ind-sort",
			// stash
			Self::IndStash => "ind-stash",
			Self::RelayStash => "relay-stash",
			// quit
			Self::KeyQuit => "key-quit",
			// which:show
			Self::IndWhichShow => "ind-which-show",
		}
	}
}

impl Display for SparkKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(self.as_ref()) }
}
