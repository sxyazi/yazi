use std::{env, fmt::Display, io::{Read, Write}, path::Path, sync::Arc};

use anyhow::{anyhow, Result};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::layout::Rect;
use tracing::warn;
use yazi_shared::{env_exists, term::Term};

use super::{Iterm2, Kitty, KittyOld};
use crate::{ueberzug::Ueberzug, Sixel, SHOWN, TMUX};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Adaptor {
	Kitty,
	KittyOld,
	Iterm2,
	Sixel,

	// Supported by Überzug++
	X11,
	Wayland,
	Chafa,
}

#[derive(Clone)]
enum Emulator {
	Unknown(Vec<Adaptor>),
	Kitty,
	Konsole,
	Iterm2,
	WezTerm,
	Foot,
	Ghostty,
	BlackBox,
	VSCode,
	Tabby,
	Hyper,
	Mintty,
	Neovim,
}

impl Adaptor {
	fn emulator() -> Emulator {
		if env_exists("NVIM_LOG_FILE") && env_exists("NVIM") {
			return Emulator::Neovim;
		}

		let vars = [
			("KITTY_WINDOW_ID", Emulator::Kitty),
			("KONSOLE_VERSION", Emulator::Konsole),
			("ITERM_SESSION_ID", Emulator::Iterm2),
			("WEZTERM_EXECUTABLE", Emulator::WezTerm),
			("GHOSTTY_RESOURCES_DIR", Emulator::Ghostty),
			("VSCODE_INJECTION", Emulator::VSCode),
			("TABBY_CONFIG_DIRECTORY", Emulator::Tabby),
		];
		match vars.into_iter().find(|v| env_exists(v.0)) {
			Some(var) => return var.1,
			None => warn!("[Adaptor] No special environment variables detected"),
		}

		let (term, program) = Self::via_env();
		match program.as_str() {
			"iTerm.app" => return Emulator::Iterm2,
			"WezTerm" => return Emulator::WezTerm,
			"ghostty" => return Emulator::Ghostty,
			"BlackBox" => return Emulator::BlackBox,
			"vscode" => return Emulator::VSCode,
			"Tabby" => return Emulator::Tabby,
			"Hyper" => return Emulator::Hyper,
			"mintty" => return Emulator::Mintty,
			_ => warn!("[Adaptor] Unknown TERM_PROGRAM: {program}"),
		}
		match term.as_str() {
			"xterm-kitty" => return Emulator::Kitty,
			"foot" => return Emulator::Foot,
			"foot-extra" => return Emulator::Foot,
			"xterm-ghostty" => return Emulator::Ghostty,
			_ => warn!("[Adaptor] Unknown TERM: {term}"),
		}

		Self::via_csi().unwrap_or(Emulator::Unknown(vec![]))
	}

	pub(super) fn detect() -> Self {
		let mut protocols = match Self::emulator() {
			Emulator::Unknown(adapters) => adapters,
			Emulator::Kitty => vec![Self::Kitty],
			Emulator::Konsole => vec![Self::KittyOld, Self::Iterm2, Self::Sixel],
			Emulator::Iterm2 => vec![Self::Iterm2, Self::Sixel],
			Emulator::WezTerm => vec![Self::Iterm2, Self::Sixel],
			Emulator::Foot => vec![Self::Sixel],
			Emulator::Ghostty => vec![Self::KittyOld],
			Emulator::BlackBox => vec![Self::Sixel],
			Emulator::VSCode => vec![Self::Sixel],
			Emulator::Tabby => vec![Self::Sixel],
			Emulator::Hyper => vec![Self::Sixel],
			Emulator::Mintty => vec![Self::Iterm2],
			Emulator::Neovim => vec![],
		};

		#[cfg(windows)]
		protocols.retain(|p| *p == Self::Iterm2);
		if env_exists("ZELLIJ_SESSION_NAME") {
			protocols.retain(|p| *p == Self::Sixel);
		}
		if *TMUX && protocols.len() > 1 {
			protocols.retain(|p| *p != Self::KittyOld);
		}
		if let Some(p) = protocols.first() {
			return *p;
		}

		match env::var("XDG_SESSION_TYPE").unwrap_or_default().as_str() {
			"x11" => return Self::X11,
			"wayland" => return Self::Wayland,
			_ => warn!("[Adaptor] Could not identify XDG_SESSION_TYPE"),
		}
		if env_exists("WAYLAND_DISPLAY") {
			return Self::Wayland;
		}
		if env_exists("DISPLAY") {
			return Self::X11;
		}
		if std::fs::symlink_metadata("/proc/sys/fs/binfmt_misc/WSLInterop").is_ok() {
			return Self::KittyOld;
		}

		warn!("[Adaptor] Falling back to chafa");
		Self::Chafa
	}

	fn via_env() -> (String, String) {
		fn tmux_env(name: &str) -> Result<String> {
			let output = std::process::Command::new("tmux").args(["show-environment", name]).output()?;

			String::from_utf8(output.stdout)?
				.trim()
				.strip_prefix(&format!("{name}="))
				.map_or_else(|| Err(anyhow!("")), |s| Ok(s.to_string()))
		}

		let mut term = env::var("TERM").unwrap_or_default();
		let mut program = env::var("TERM_PROGRAM").unwrap_or_default();

		if *TMUX {
			term = tmux_env("TERM").unwrap_or(term);
			program = tmux_env("TERM_PROGRAM").unwrap_or(program);
		}

		(term, program)
	}

	fn via_csi() -> Result<Emulator> {
		enable_raw_mode()?;
		std::io::stdout().write_all(b"\x1b[>q\x1b_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\\x1b[c")?;
		std::io::stdout().flush()?;

		let mut stdin = std::io::stdin().lock();
		let mut buf = String::with_capacity(200);
		loop {
			let mut c = [0; 1];
			if stdin.read(&mut c)? == 0 {
				break;
			}
			if c[0] == b'c' && buf.contains("\x1b[?") {
				break;
			}
			buf.push(c[0] as char);
		}

		disable_raw_mode().ok();
		let names = [
			("kitty", Emulator::Kitty),
			("Konsole", Emulator::Konsole),
			("iTerm2", Emulator::Iterm2),
			("WezTerm", Emulator::WezTerm),
			("foot", Emulator::Foot),
			("ghostty", Emulator::Ghostty),
		];

		for (name, emulator) in names.iter() {
			if buf.contains(name) {
				return Ok(emulator.clone());
			}
		}

		let mut adapters = Vec::with_capacity(2);
		if buf.contains("\x1b_Gi=31;OK") {
			adapters.push(Adaptor::KittyOld);
		}
		if ["?4;", "?4c", ";4;", ";4c"].iter().any(|s| buf.contains(s)) {
			adapters.push(Adaptor::Sixel);
		}

		Ok(Emulator::Unknown(adapters))
	}
}

impl Display for Adaptor {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Kitty => write!(f, "kitty"),
			Self::KittyOld => write!(f, "kitty"),
			Self::Iterm2 => write!(f, "iterm2"),
			Self::Sixel => write!(f, "sixel"),
			Self::X11 => write!(f, "x11"),
			Self::Wayland => write!(f, "wayland"),
			Self::Chafa => write!(f, "chafa"),
		}
	}
}

impl Adaptor {
	pub(super) fn start(self) { Ueberzug::start(self); }

	pub async fn image_show(self, path: &Path, rect: Rect) -> Result<(u32, u32)> {
		match self {
			Self::Kitty => Kitty::image_show(path, rect).await,
			Self::KittyOld => KittyOld::image_show(path, rect).await,
			Self::Iterm2 => Iterm2::image_show(path, rect).await,
			Self::Sixel => Sixel::image_show(path, rect).await,
			_ => Ueberzug::image_show(path, rect).await,
		}
	}

	pub fn image_hide(self) -> Result<()> {
		if let Some(rect) = SHOWN.swap(None) { self.image_erase(*rect) } else { Ok(()) }
	}

	pub fn image_erase(self, rect: Rect) -> Result<()> {
		match self {
			Self::Kitty => Kitty::image_erase(rect),
			Self::Iterm2 => Iterm2::image_erase(rect),
			Self::KittyOld => KittyOld::image_erase(),
			Self::Sixel => Sixel::image_erase(rect),
			_ => Ueberzug::image_erase(rect),
		}
	}

	#[inline]
	pub fn shown_load(self) -> Option<Rect> { SHOWN.load_full().map(|r| *r) }

	#[inline]
	pub(super) fn shown_store(rect: Rect, size: (u32, u32)) {
		SHOWN.store(Some(Arc::new(
			Term::ratio()
				.map(|(r1, r2)| Rect {
					x:      rect.x,
					y:      rect.y,
					width:  (size.0 as f64 / r1).ceil() as u16,
					height: (size.1 as f64 / r2).ceil() as u16,
				})
				.unwrap_or(rect),
		)));
	}

	#[inline]
	pub(super) fn needs_ueberzug(self) -> bool {
		!matches!(self, Self::Kitty | Self::KittyOld | Self::Iterm2 | Self::Sixel)
	}
}
