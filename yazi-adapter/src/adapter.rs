use std::{env, fmt::Display, path::Path, sync::Arc};

use anyhow::Result;
use ratatui::layout::Rect;
use tracing::warn;
use yazi_shared::env_exists;

use super::{Iterm2, Kitty, KittyOld};
use crate::{Chafa, Emulator, Sixel, Ueberzug, SHOWN, TMUX};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Adapter {
	Kitty,
	KittyOld,
	Iterm2,
	Sixel,

	// Supported by Überzug++
	X11,
	Wayland,
	Chafa,
}

impl Display for Adapter {
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

impl Adapter {
	pub async fn image_show(self, path: &Path, max: Rect) -> Result<Rect> {
		if max.is_empty() {
			return Ok(Rect::default());
		}

		match self {
			Self::Kitty => Kitty::image_show(path, max).await,
			Self::KittyOld => KittyOld::image_show(path, max).await,
			Self::Iterm2 => Iterm2::image_show(path, max).await,
			Self::Sixel => Sixel::image_show(path, max).await,
			Self::X11 | Self::Wayland => Ueberzug::image_show(path, max).await,
			Self::Chafa => Chafa::image_show(path, max).await,
		}
	}

	pub fn image_hide(self) -> Result<()> {
		if let Some(area) = SHOWN.swap(None) { self.image_erase(*area) } else { Ok(()) }
	}

	pub fn image_erase(self, area: Rect) -> Result<()> {
		match self {
			Self::Kitty => Kitty::image_erase(area),
			Self::KittyOld => KittyOld::image_erase(area),
			Self::Iterm2 => Iterm2::image_erase(area),
			Self::Sixel => Sixel::image_erase(area),
			Self::X11 | Self::Wayland => Ueberzug::image_erase(area),
			Self::Chafa => Chafa::image_erase(area),
		}
	}

	#[inline]
	pub fn shown_load(self) -> Option<Rect> { SHOWN.load_full().map(|r| *r) }

	#[inline]
	pub(super) fn shown_store(area: Rect) { SHOWN.store(Some(Arc::new(area))); }

	pub(super) fn start(self) { Ueberzug::start(self); }

	#[inline]
	pub(super) fn needs_ueberzug(self) -> bool {
		!matches!(self, Self::Kitty | Self::KittyOld | Self::Iterm2 | Self::Sixel)
	}
}

impl Adapter {
	pub fn matches() -> Self {
		let mut protocols = Emulator::detect().adapters();

		#[cfg(windows)]
		protocols.retain(|p| *p == Self::Iterm2);
		if env_exists("ZELLIJ_SESSION_NAME") {
			protocols.retain(|p| *p == Self::Sixel);
		} else if *TMUX {
			protocols.retain(|p| *p != Self::KittyOld);
		}
		if let Some(p) = protocols.first() {
			return *p;
		}

		match env::var("XDG_SESSION_TYPE").unwrap_or_default().as_str() {
			"x11" => return Self::X11,
			"wayland" => return Self::Wayland,
			_ => warn!("[Adapter] Could not identify XDG_SESSION_TYPE"),
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

		warn!("[Adapter] Falling back to chafa");
		Self::Chafa
	}
}
