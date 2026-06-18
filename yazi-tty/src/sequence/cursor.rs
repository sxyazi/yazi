use std::fmt::{self, Display};

/// Move cursor to 0-based (col, row) position
pub struct MoveTo(pub u16, pub u16);

impl Display for MoveTo {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "\x1b[{};{}H", self.1 + 1, self.0 + 1)
	}
}

/// Show cursor
pub struct ShowCursor;

impl Display for ShowCursor {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("\x1b[?25h") }
}

/// Hide cursor
pub struct HideCursor;

impl Display for HideCursor {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("\x1b[?25l") }
}

/// Save cursor position
pub struct SaveCursorPos;

impl Display for SaveCursorPos {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("\x1b[s") }
}

/// Restore cursor position
pub struct RestoreCursorPos;

impl Display for RestoreCursorPos {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("\x1b[u") }
}

/// Set terminal window title
pub struct SetTitle<'a>(pub &'a str);

impl Display for SetTitle<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "\x1b]2;{}\x1b\\", self.0) }
}

/// Set cursor style
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SetCursorStyle {
	#[default]
	Default           = 0,
	BlinkingBlock     = 1,
	SteadyBlock       = 2,
	BlinkingUnderline = 3,
	SteadyUnderline   = 4,
	BlinkingBar       = 5,
	SteadyBar         = 6,
}

impl Display for SetCursorStyle {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "\x1b[{} q", *self as u8) }
}

/// Restore cursor shape and blink state
pub struct RestoreCursorStyle {
	pub shape: u8,
	pub blink: bool,
}

impl Display for RestoreCursorStyle {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let (shape, shape_blink) = match self.shape {
			u8::MAX => (0, None),
			n => (n.max(1).div_ceil(2), Some(n.max(1) & 1 == 1)),
		};

		let blink = shape_blink.unwrap_or(self.blink);
		match shape {
			2 if blink => SetCursorStyle::BlinkingUnderline,
			2 if !blink => SetCursorStyle::SteadyUnderline,
			3 if blink => SetCursorStyle::BlinkingBar,
			3 if !blink => SetCursorStyle::SteadyBar,
			_ if blink => SetCursorStyle::Default,
			_ if !blink => SetCursorStyle::SteadyBlock,
			_ => unreachable!(),
		}
		.fmt(f)
	}
}
