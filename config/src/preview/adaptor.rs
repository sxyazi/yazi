use std::env;

#[derive(Debug, PartialEq, Eq)]
pub enum PreviewAdaptor {
	Kitty,
	Iterm2,
	Sixel,

	// Supported by Überzug++
	X11,
	Wayland,
	Chafa,
}

impl Default for PreviewAdaptor {
	fn default() -> Self {
		if env::var("KITTY_WINDOW_ID").is_ok() {
			return Self::Kitty;
		}
		if env::var("KONSOLE_VERSION").is_ok() {
			return Self::Kitty;
		}
		match env::var("TERM").unwrap_or_default().as_str() {
			"xterm-kitty" => return Self::Kitty,
			"wezterm" => return Self::Kitty,
			"foot" => return Self::Sixel,
			_ => {}
		}
		match env::var("TERM_PROGRAM").unwrap_or_default().as_str() {
			"iTerm.app" => return Self::Iterm2,
			"Hyper" => return Self::Sixel,
			_ => {}
		}
		match env::var("XDG_SESSION_TYPE").unwrap_or_default().as_str() {
			"x11" => return Self::X11,
			"wayland" => return Self::Wayland,
			_ => Self::Chafa,
		}
	}
}

impl ToString for PreviewAdaptor {
	fn to_string(&self) -> String {
		match self {
			PreviewAdaptor::Kitty => "kitty",
			PreviewAdaptor::Iterm2 => "iterm2",
			PreviewAdaptor::Sixel => "sixel",
			PreviewAdaptor::X11 => "x11",
			PreviewAdaptor::Wayland => "wayland",
			PreviewAdaptor::Chafa => "chafa",
		}
		.to_string()
	}
}

impl PreviewAdaptor {
	#[inline]
	pub fn needs_ueberzug(&self) -> bool {
		match self {
			PreviewAdaptor::Kitty => false,
			PreviewAdaptor::Iterm2 => false,
			PreviewAdaptor::Sixel => false,
			_ => true,
		}
	}
}
