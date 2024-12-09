use std::{env, fmt::Display, path::Path};

use anyhow::Result;
use ratatui::layout::Rect;
use tracing::warn;
use yazi_shared::env_exists;

use crate::{Brand, Emulator, Offset, SHOWN, TMUX, WSL, drivers};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Adapter {
	Kgp,
	KgpOld,
	Iip,
	Sixel,

	// Supported by Überzug++
	X11,
	Wayland,
	Chafa,
}

impl Display for Adapter {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Kgp => write!(f, "kgp"),
			Self::KgpOld => write!(f, "kgp-old"),
			Self::Iip => write!(f, "iip"),
			Self::Sixel => write!(f, "sixel"),
			Self::X11 => write!(f, "x11"),
			Self::Wayland => write!(f, "wayland"),
			Self::Chafa => write!(f, "chafa"),
		}
	}
}

impl Adapter {
	pub async fn image_show(self, path: &Path, max: Rect, offset: Option<Offset>) -> Result<Rect> {
		if max.is_empty() {
			return Ok(Rect::default());
		}

		match self {
			Self::Kgp => drivers::Kgp::image_show(path, max, offset).await,
			Self::KgpOld => drivers::KgpOld::image_show(path, max, offset).await,
			Self::Iip => drivers::Iip::image_show(path, max, offset).await,
			Self::Sixel => drivers::Sixel::image_show(path, max, offset).await,
			Self::X11 | Self::Wayland => drivers::Ueberzug::image_show(path, max, offset).await,
			Self::Chafa => drivers::Chafa::image_show(path, max, offset).await,
		}
	}

	pub fn image_hide(self) -> Result<()> {
		if let Some(area) = SHOWN.replace(None) { self.image_erase(area) } else { Ok(()) }
	}

	pub fn image_erase(self, area: Rect) -> Result<()> {
		match self {
			Self::Kgp => drivers::Kgp::image_erase(area),
			Self::KgpOld => drivers::KgpOld::image_erase(area),
			Self::Iip => drivers::Iip::image_erase(area),
			Self::Sixel => drivers::Sixel::image_erase(area),
			Self::X11 | Self::Wayland => drivers::Ueberzug::image_erase(area),
			Self::Chafa => drivers::Chafa::image_erase(area),
		}
	}

	#[inline]
	pub fn shown_load(self) -> Option<Rect> { SHOWN.get() }

	#[inline]
	pub(super) fn shown_store(area: Rect) { SHOWN.set(Some(area)); }

	pub(super) fn start(self) { drivers::Ueberzug::start(self); }

	#[inline]
	pub(super) fn needs_ueberzug(self) -> bool {
		!matches!(self, Self::Kgp | Self::KgpOld | Self::Iip | Self::Sixel)
	}
}

impl Adapter {
	pub fn matches(emulator: Emulator) -> Self {
		if matches!(emulator.kind.left(), Some(Brand::Microsoft)) {
			return Self::Sixel;
		} else if *WSL && matches!(emulator.kind.left(), Some(Brand::WezTerm)) {
			return Self::KgpOld;
		}

		let mut protocols = emulator.adapters().to_owned();
		#[cfg(windows)]
		protocols.retain(|p| *p == Self::Iip);
		if env_exists("ZELLIJ_SESSION_NAME") {
			protocols.retain(|p| *p == Self::Sixel);
		} else if *TMUX != 0 {
			protocols.retain(|p| *p != Self::KgpOld);
		}
		if let Some(p) = protocols.first() {
			return *p;
		}

		let supported_compositor = drivers::Ueberzug::supported_compositor();
		match env::var("XDG_SESSION_TYPE").unwrap_or_default().as_str() {
			"x11" => return Self::X11,
			"wayland" if supported_compositor => return Self::Wayland,
			"wayland" if !supported_compositor => return Self::Chafa,
			_ => warn!("[Adapter] Could not identify XDG_SESSION_TYPE"),
		}
		if env_exists("WAYLAND_DISPLAY") {
			return if supported_compositor { Self::Wayland } else { Self::Chafa };
		}
		if env_exists("DISPLAY") {
			return Self::X11;
		}

		warn!("[Adapter] Falling back to chafa");
		Self::Chafa
	}
}
