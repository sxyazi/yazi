use std::fmt::{self, Display};

/// Begin synchronized update (DEC 2026)
pub struct BeginSyncUpdate;

impl Display for BeginSyncUpdate {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("\x1b[?2026h") }
}

/// End synchronized update (DEC 2026)
pub struct EndSyncUpdate;

impl Display for EndSyncUpdate {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("\x1b[?2026l") }
}
