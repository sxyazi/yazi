use std::fmt::Display;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SparkKind {
	// app:title
	IndAppTitle,

	// mgr:sort
	KeySort,
	IndSort,
	// mgr:stash
	IndStash,
	RelayStash,
	// mgr:quit
	KeyQuit,

	// which:activate
	IndWhichActivate,

	// notify:push
	RelayNotifyPush,
}

impl AsRef<str> for SparkKind {
	fn as_ref(&self) -> &str {
		match self {
			// app:title
			Self::IndAppTitle => "ind-app-title",

			// mgr:sort
			Self::KeySort => "key-sort",
			Self::IndSort => "ind-sort",
			// mgr:stash
			Self::IndStash => "ind-stash",
			Self::RelayStash => "relay-stash",
			// mgr:quit
			Self::KeyQuit => "key-quit",

			// which:activate
			Self::IndWhichActivate => "ind-which-activate",

			// notify:push
			Self::RelayNotifyPush => "relay-notify-push",
		}
	}
}

impl Display for SparkKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(self.as_ref()) }
}
