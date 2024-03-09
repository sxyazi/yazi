use std::{env, fmt::Display, path::Path, sync::Arc};

use anyhow::Result;
use ratatui::layout::Rect;
use tracing::warn;
use yazi_shared::{env_exists, term::Term};

use super::{Iterm2, Kitty, KittyOld};
use crate::{ueberzug::Ueberzug, Emulator, Sixel, SHOWN, TMUX};

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

	pub(super) fn start(self) { Ueberzug::start(self); }

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

impl Adaptor {
	pub fn matches() -> Self {
		let mut protocols = Emulator::detect().adapters();

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
}
