use std::env;

#[derive(Debug, PartialEq, Eq)]
pub enum PreviewAdaptor {
	Kitty,
	Iterm2,

	// Supported by Überzug++
	X11,
	Wayland,
	Sixel,
	Chafa,
}

impl Default for PreviewAdaptor {
	fn default() -> Self {
		match env::var("TERM").unwrap_or_default().as_str() {
			"wezterm" => return Self::Kitty,
			"xterm-kitty" => return Self::Kitty,
			"iterm2" => return Self::Iterm2,
			"foot" => return Self::Sixel,
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
			PreviewAdaptor::X11 => "x11",
			PreviewAdaptor::Wayland => "wayland",
			PreviewAdaptor::Sixel => "sixel",
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
			_ => true,
		}
	}
}
