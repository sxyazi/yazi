use std::fmt::{self, Display};

use ratatui_core::backend::ClearType;

/// Erase in Display (ED)
pub enum EraseDisplay {
	AfterCursor,
	BeforeCursor,
	All,
	SavedLines,
}

impl Display for EraseDisplay {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::AfterCursor => f.write_str("\x1b[0J"),
			Self::BeforeCursor => f.write_str("\x1b[1J"),
			Self::All => f.write_str("\x1b[2J"),
			Self::SavedLines => f.write_str("\x1b[3J"),
		}
	}
}

/// Erase in Line (EL)
pub enum EraseLine {
	AfterCursor,
	BeforeCursor,
	All,
}

impl Display for EraseLine {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::AfterCursor => f.write_str("\x1b[0K"),
			Self::BeforeCursor => f.write_str("\x1b[1K"),
			Self::All => f.write_str("\x1b[2K"),
		}
	}
}

/// Erase specified region of the display
pub struct EraseRegion(pub ClearType);

impl Display for EraseRegion {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self.0 {
			ClearType::All => EraseDisplay::All.fmt(f),
			ClearType::AfterCursor => EraseDisplay::AfterCursor.fmt(f),
			ClearType::BeforeCursor => EraseDisplay::BeforeCursor.fmt(f),
			ClearType::CurrentLine => EraseLine::All.fmt(f),
			ClearType::UntilNewLine => EraseLine::AfterCursor.fmt(f),
		}
	}
}
