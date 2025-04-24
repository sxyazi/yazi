use tracing::debug;
use yazi_shared::env_exists;

use crate::Mux;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
			("tmux ", Self::Tmux),
			("libvterm", Self::VTerm),
			("Bobcat", Self::Bobcat),
		];
		names.into_iter().find(|&(n, _)| resp.contains(n)).map(|(_, b)| b)
	}

	pub fn from_env() -> Option<Self> {
		use Brand as B;

		let (term, program) = B::env();
		let vars = [
			("KITTY_WINDOW_ID", B::Kitty),
			("KONSOLE_VERSION", B::Konsole),
			("ITERM_SESSION_ID", B::Iterm2),
			("WEZTERM_EXECUTABLE", B::WezTerm),
			("GHOSTTY_RESOURCES_DIR", B::Ghostty),
			("WT_Session", B::Microsoft),
			("WARP_HONOR_PS1", B::Warp),
			("VSCODE_INJECTION", B::VSCode),
			("TABBY_CONFIG_DIRECTORY", B::Tabby),
		];

		match term.as_str() {
			"xterm-kitty" => return Some(B::Kitty),
			"foot" => return Some(B::Foot),
			"foot-extra" => return Some(B::Foot),
			"xterm-ghostty" => return Some(B::Ghostty),
			"rio" => return Some(B::Rio),
			"rxvt-unicode-256color" => return Some(B::Urxvt),
			_ => {}
		}
		match program.as_str() {
			"iTerm.app" => return Some(B::Iterm2),
			"WezTerm" => return Some(B::WezTerm),
			"ghostty" => return Some(B::Ghostty),
			"WarpTerminal" => return Some(B::Warp),
			"rio" => return Some(B::Rio),
			"BlackBox" => return Some(B::BlackBox),
			"vscode" => return Some(B::VSCode),
			"Tabby" => return Some(B::Tabby),
			"Hyper" => return Some(B::Hyper),
			"mintty" => return Some(B::Mintty),
			"Apple_Terminal" => return Some(B::Apple),
			_ => {}
		}
		if let Some((var, brand)) = vars.into_iter().find(|&(s, _)| env_exists(s)) {
			debug!("Detected special environment variable: {var}");
			return Some(brand);
		}

		None
	}

	pub(super) fn adapters(self) -> &'static [crate::Adapter] {
		use Brand as B;

		use crate::Adapter as A;

		match self {
			B::Kitty => &[A::Kgp],
			B::Konsole => &[A::KgpOld],
			B::Iterm2 => &[A::Iip, A::Sixel],
			B::WezTerm => &[A::Iip, A::Sixel],
			B::Foot => &[A::Sixel],
			B::Ghostty => &[A::Kgp],
			B::Microsoft => &[A::Sixel],
			B::Warp => &[A::Iip, A::KgpOld],
			B::Rio => &[A::Iip, A::Sixel],
			B::BlackBox => &[A::Sixel],
			B::VSCode => &[A::Iip, A::Sixel],
			B::Tabby => &[A::Iip, A::Sixel],
			B::Hyper => &[A::Iip, A::Sixel],
			B::Mintty => &[A::Iip],
			B::Tmux => &[],
			B::VTerm => &[],
			B::Apple => &[],
			B::Urxvt => &[],
			B::Bobcat => &[A::Iip, A::Sixel],
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
