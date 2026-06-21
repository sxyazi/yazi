use std::fmt::{self, Display};

/// Enter alternate screen buffer
pub struct EnterAlternateScreen;

impl Display for EnterAlternateScreen {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("\x1b[?1049h") }
}

/// Leave alternate screen buffer
pub struct LeaveAlternateScreen;

impl Display for LeaveAlternateScreen {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("\x1b[?1049l") }
}

/// Enable bracketed paste mode
pub struct EnableBracketedPaste;

impl Display for EnableBracketedPaste {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("\x1b[?2004h") }
}

/// Disable bracketed paste mode
pub struct DisableBracketedPaste;

impl Display for DisableBracketedPaste {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("\x1b[?2004l") }
}

/// Enable focus change reporting
pub struct EnableFocusChange;

impl Display for EnableFocusChange {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("\x1b[?1004h") }
}

/// Disable focus change reporting
pub struct DisableFocusChange;

impl Display for DisableFocusChange {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("\x1b[?1004l") }
}

/// Enable mouse capture (X10 + ButtonEvent + UrXvt + SGR)
pub struct EnableMouseCapture;

impl Display for EnableMouseCapture {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str("\x1b[?1000h\x1b[?1002h\x1b[?1015h\x1b[?1006h")
	}
}

/// Disable mouse capture (SGR + UrXvt + ButtonEvent + X10)
pub struct DisableMouseCapture;

impl Display for DisableMouseCapture {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str("\x1b[?1006l\x1b[?1015l\x1b[?1002l\x1b[?1000l")
	}
}
