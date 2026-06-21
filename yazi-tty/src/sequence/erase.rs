use std::fmt::{self, Display};

use ratatui_core::backend::ClearType;

/// Erase entire display
pub struct EraseScreen;

impl Display for EraseScreen {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { EraseRegion(ClearType::All).fmt(f) }
}

/// Erase specified region of the display
pub struct EraseRegion(pub ClearType);

impl Display for EraseRegion {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self.0 {
			ClearType::All => f.write_str("\x1b[2J"),
			ClearType::AfterCursor => f.write_str("\x1b[0J"),
			ClearType::BeforeCursor => f.write_str("\x1b[1J"),
			ClearType::CurrentLine => f.write_str("\x1b[2K"),
			ClearType::UntilNewLine => f.write_str("\x1b[0K"),
		}
	}
}
