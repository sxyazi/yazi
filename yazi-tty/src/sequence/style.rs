use std::fmt::{self, Display};

use ratatui_core::style::Color;

/// Set foreground color from a `Color`.
pub struct SetFg(pub Color);

impl Display for SetFg {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self.0 {
			Color::Reset => f.write_str("\x1b[39m"),
			Color::Black => f.write_str("\x1b[30m"),
			Color::Red => f.write_str("\x1b[31m"),
			Color::Green => f.write_str("\x1b[32m"),
			Color::Yellow => f.write_str("\x1b[33m"),
			Color::Blue => f.write_str("\x1b[34m"),
			Color::Magenta => f.write_str("\x1b[35m"),
			Color::Cyan => f.write_str("\x1b[36m"),
			Color::Gray => f.write_str("\x1b[37m"),
			Color::DarkGray => f.write_str("\x1b[90m"),
			Color::LightRed => f.write_str("\x1b[91m"),
			Color::LightGreen => f.write_str("\x1b[92m"),
			Color::LightYellow => f.write_str("\x1b[93m"),
			Color::LightBlue => f.write_str("\x1b[94m"),
			Color::LightMagenta => f.write_str("\x1b[95m"),
			Color::LightCyan => f.write_str("\x1b[96m"),
			Color::White => f.write_str("\x1b[97m"),
			Color::Rgb(r, g, b) => write!(f, "\x1b[38;2;{r};{g};{b}m"),
			Color::Indexed(n) => write!(f, "\x1b[38;5;{n}m"),
		}
	}
}

/// Set background color from a `Color`.
pub struct SetBg(pub Color);

impl Display for SetBg {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self.0 {
			Color::Reset => f.write_str("\x1b[49m"),
			Color::Black => f.write_str("\x1b[40m"),
			Color::Red => f.write_str("\x1b[41m"),
			Color::Green => f.write_str("\x1b[42m"),
			Color::Yellow => f.write_str("\x1b[43m"),
			Color::Blue => f.write_str("\x1b[44m"),
			Color::Magenta => f.write_str("\x1b[45m"),
			Color::Cyan => f.write_str("\x1b[46m"),
			Color::Gray => f.write_str("\x1b[47m"),
			Color::DarkGray => f.write_str("\x1b[100m"),
			Color::LightRed => f.write_str("\x1b[101m"),
			Color::LightGreen => f.write_str("\x1b[102m"),
			Color::LightYellow => f.write_str("\x1b[103m"),
			Color::LightBlue => f.write_str("\x1b[104m"),
			Color::LightMagenta => f.write_str("\x1b[105m"),
			Color::LightCyan => f.write_str("\x1b[106m"),
			Color::White => f.write_str("\x1b[107m"),
			Color::Rgb(r, g, b) => write!(f, "\x1b[48;2;{r};{g};{b}m"),
			Color::Indexed(n) => write!(f, "\x1b[48;5;{n}m"),
		}
	}
}

/// Set underline color from a `Color`.
pub struct SetUnderlineColor(pub Color);

impl Display for SetUnderlineColor {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self.0 {
			Color::Reset => f.write_str("\x1b[59m"),
			Color::Black => write!(f, "\x1b[58;5;0m"),
			Color::Red => write!(f, "\x1b[58;5;1m"),
			Color::Green => write!(f, "\x1b[58;5;2m"),
			Color::Yellow => write!(f, "\x1b[58;5;3m"),
			Color::Blue => write!(f, "\x1b[58;5;4m"),
			Color::Magenta => write!(f, "\x1b[58;5;5m"),
			Color::Cyan => write!(f, "\x1b[58;5;6m"),
			Color::Gray => write!(f, "\x1b[58;5;7m"),
			Color::DarkGray => write!(f, "\x1b[58;5;8m"),
			Color::LightRed => write!(f, "\x1b[58;5;9m"),
			Color::LightGreen => write!(f, "\x1b[58;5;10m"),
			Color::LightYellow => write!(f, "\x1b[58;5;11m"),
			Color::LightBlue => write!(f, "\x1b[58;5;12m"),
			Color::LightMagenta => write!(f, "\x1b[58;5;13m"),
			Color::LightCyan => write!(f, "\x1b[58;5;14m"),
			Color::White => write!(f, "\x1b[58;5;15m"),
			Color::Rgb(r, g, b) => write!(f, "\x1b[58;2;{r};{g};{b}m"),
			Color::Indexed(n) => write!(f, "\x1b[58;5;{n}m"),
		}
	}
}

/// SGR text attribute.
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum SetSgr {
	Reset           = 0,
	Bold            = 1,
	Dim             = 2,
	Italic          = 3,
	Underlined      = 4,
	SlowBlink       = 5,
	RapidBlink      = 6,
	Reverse         = 7,
	Hidden          = 8,
	CrossedOut      = 9,
	NormalIntensity = 22,
	NoItalic        = 23,
	NoUnderline     = 24,
	NoBlink         = 25,
	NoReverse       = 27,
	NoHidden        = 28,
	NotCrossedOut   = 29,
}

impl Display for SetSgr {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "\x1b[{}m", *self as u8) }
}

/// Reset all colors and SGR attributes
pub struct ResetAttrs;

impl Display for ResetAttrs {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}{}{}", SetFg(Color::Reset), SetBg(Color::Reset), SetSgr::Reset)
	}
}
