use std::{env, path::Path, sync::Arc};

use anyhow::{anyhow, Result};
use ratatui::prelude::Rect;
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

#[derive(Clone, Copy)]
enum Emulator {
	Unknown,
	Kitty,
	Konsole,
	Iterm2,
	WezTerm,
	Foot,
	BlackBox,
	VSCode,
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
			("VSCODE_INJECTION", Emulator::VSCode),
		];
		match vars.into_iter().find(|v| env_exists(v.0)) {
			Some(var) => return var.1,
			None => warn!("[Adaptor] No special environment variables detected"),
		}

		let (term, program) = Self::term_program();
		match program.as_str() {
			"iTerm.app" => return Emulator::Iterm2,
			"WezTerm" => return Emulator::WezTerm,
			"BlackBox" => return Emulator::BlackBox,
			"vscode" => return Emulator::VSCode,
			"Hyper" => return Emulator::Hyper,
			"mintty" => return Emulator::Mintty,
			_ => warn!("[Adaptor] Unknown TERM_PROGRAM: {program}"),
		}
		match term.as_str() {
			"xterm-kitty" => return Emulator::Kitty,
			"foot" => return Emulator::Foot,
			"foot-extra" => return Emulator::Foot,
			_ => warn!("[Adaptor] Unknown TERM: {term}"),
		}
		Emulator::Unknown
	}

	pub(super) fn detect() -> Self {
		let mut protocols = match Self::emulator() {
			Emulator::Unknown => vec![],
			Emulator::Kitty => vec![Self::Kitty],
			Emulator::Konsole => vec![Self::KittyOld, Self::Iterm2, Self::Sixel],
			Emulator::Iterm2 => vec![Self::Iterm2, Self::Sixel],
			Emulator::WezTerm => vec![Self::Iterm2, Self::Sixel],
			Emulator::Foot => vec![Self::Sixel],
			Emulator::BlackBox => vec![Self::Sixel],
			Emulator::VSCode => vec![Self::Sixel],
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
			return Self::Kitty;
		}

		warn!("[Adaptor] Falling back to chafa");
		Self::Chafa
	}

	pub(super) fn term_program() -> (String, String) {
		fn tmux_env(name: &str) -> Result<String> {
			let output = std::process::Command::new("tmux").args(["show-environment", name]).output()?;

			String::from_utf8(output.stdout)?
				.trim()
				.strip_prefix(&format!("{}=", name))
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
}

impl ToString for Adaptor {
	fn to_string(&self) -> String {
		match self {
			Self::Kitty => "kitty",
			Self::KittyOld => "kitty",
			Self::Iterm2 => "iterm2",
			Self::Sixel => "sixel",
			Self::X11 => "x11",
			Self::Wayland => "wayland",
			Self::Chafa => "chafa",
		}
		.to_string()
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
