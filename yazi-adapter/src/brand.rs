use tracing::debug;
use yazi_shared::env_exists;

use crate::Mux;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Brand {
	Kitty,
	Konsole,
	Iterm2,
	WezTerm,
	Foot,
	Ghostty,
	Microsoft,
	Warp,
	Rio,
	BlackBox,
	VSCode,
	Tabby,
	Hyper,
	Mintty,
	Tmux,
	VTerm,
	Apple,
	Urxvt,
	Bobcat,
}

impl Brand {
	pub(super) fn from_csi(resp: &str) -> Option<Self> {
		let names = [
			("kitty", Self::Kitty),
			("Konsole", Self::Konsole),
			("iTerm2", Self::Iterm2),
			("WezTerm", Self::WezTerm),
			("foot", Self::Foot),
			("ghostty", Self::Ghostty),
			("Warp", Self::Warp),
			("tmux ", Self::Tmux),
			("libvterm", Self::VTerm),
			("Bobcat", Self::Bobcat),
		];
		names.into_iter().find(|&(n, _)| resp.contains(n)).map(|(_, b)| b)
	}

	pub fn from_env() -> Option<Self> {
		let (term, program) = Self::env();
		let vars = [
			("KITTY_WINDOW_ID", Self::Kitty),
			("KONSOLE_VERSION", Self::Konsole),
			("ITERM_SESSION_ID", Self::Iterm2),
			("WEZTERM_EXECUTABLE", Self::WezTerm),
			("GHOSTTY_RESOURCES_DIR", Self::Ghostty),
			("WT_Session", Self::Microsoft),
			("WARP_HONOR_PS1", Self::Warp),
			("VSCODE_INJECTION", Self::VSCode),
			("TABBY_CONFIG_DIRECTORY", Self::Tabby),
		];

		match term.as_str() {
			"xterm-kitty" => return Some(Self::Kitty),
			"foot" => return Some(Self::Foot),
			"foot-extra" => return Some(Self::Foot),
			"xterm-ghostty" => return Some(Self::Ghostty),
			"rio" => return Some(Self::Rio),
			"rxvt-unicode-256color" => return Some(Self::Urxvt),
			_ => {}
		}
		match program.as_str() {
			"iTerm.app" => return Some(Self::Iterm2),
			"WezTerm" => return Some(Self::WezTerm),
			"ghostty" => return Some(Self::Ghostty),
			"WarpTerminal" => return Some(Self::Warp),
			"rio" => return Some(Self::Rio),
			"BlackBox" => return Some(Self::BlackBox),
			"vscode" => return Some(Self::VSCode),
			"Tabby" => return Some(Self::Tabby),
			"Hyper" => return Some(Self::Hyper),
			"mintty" => return Some(Self::Mintty),
			"Apple_Terminal" => return Some(Self::Apple),
			_ => {}
		}
		if let Some((var, brand)) = vars.into_iter().find(|&(s, _)| env_exists(s)) {
			debug!("Detected special environment variable: {var}");
			return Some(brand);
		}

		None
	}

	pub(super) fn adapters(self) -> &'static [crate::Adapter] {
		use crate::Adapter as A;

		match self {
			Self::Kitty => &[A::Kgp],
			Self::Konsole => &[A::KgpOld],
			Self::Iterm2 => &[A::Iip, A::Sixel],
			Self::WezTerm => &[A::Iip, A::Sixel],
			Self::Foot => &[A::Sixel],
			Self::Ghostty => &[A::Kgp],
			Self::Microsoft => &[A::Sixel],
			Self::Warp => &[A::Iip, A::KgpOld],
			Self::Rio => &[A::Iip, A::Sixel],
			Self::BlackBox => &[A::Sixel],
			Self::VSCode => &[A::Iip, A::Sixel],
			Self::Tabby => &[A::Iip, A::Sixel],
			Self::Hyper => &[A::Iip, A::Sixel],
			Self::Mintty => &[A::Iip],
			Self::Tmux => &[],
			Self::VTerm => &[],
			Self::Apple => &[],
			Self::Urxvt => &[],
			Self::Bobcat => &[A::Iip, A::Sixel],
		}
	}

	fn env() -> (String, String) {
		let (term, program) = Mux::term_program();
		(
			term.unwrap_or(std::env::var("TERM").unwrap_or_default()),
			program.unwrap_or(std::env::var("TERM_PROGRAM").unwrap_or_default()),
		)
	}
}
