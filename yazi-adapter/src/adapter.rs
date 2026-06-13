use std::{env, path::PathBuf};

use anyhow::Result;
use ratatui::layout::Rect;
use strum::{Display, IntoStaticStr};
use tracing::warn;
use yazi_emulator::{Emulator, TMUX};
use yazi_shared::env_exists;

use crate::{Adapters, SHOWN, drivers};

const ZELLIJ_SESSION_NAME_ENV: &str = "ZELLIJ_SESSION_NAME";
pub const ZELLIJ_KITTY_PASSTHROUGH_ENV: &str = "YAZI_ZELLIJ_KITTY_PASSTHROUGH";

#[derive(Clone, Copy, Debug, Display, Eq, IntoStaticStr, PartialEq)]
#[strum(serialize_all = "kebab-case")]
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

impl Adapter {
	pub async fn image_show<P>(self, path: P, max: Rect) -> Result<Rect>
	where
		P: Into<PathBuf>,
	{
		if max.is_empty() {
			return Ok(Rect::default());
		}

		let path = path.into();
		match self {
			Self::Kgp => drivers::Kgp::image_show(path, max).await,
			Self::KgpOld => drivers::KgpOld::image_show(path, max).await,
			Self::Iip => drivers::Iip::image_show(path, max).await,
			Self::Sixel => drivers::Sixel::image_show(path, max).await,
			Self::X11 | Self::Wayland => drivers::Ueberzug::image_show(path, max).await,
			Self::Chafa => drivers::Chafa::image_show(path, max).await,
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
	pub fn matches(emulator: &Emulator) -> Self {
		let mut adapters: Adapters = emulator.into();
		if env_exists(ZELLIJ_SESSION_NAME_ENV) {
			adapters.retain(|p| p.supported_in_zellij());
		} else if TMUX.get() {
			adapters.retain(|p| *p != Self::KgpOld);
		}
		if let Some(p) = adapters.first() {
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
		match env::var("DISPLAY").unwrap_or_default().as_str() {
			s if !s.is_empty() && !s.contains("/org.xquartz") => return Self::X11,
			_ => {}
		}

		warn!("[Adapter] Falling back to chafa");
		Self::Chafa
	}

	fn supported_in_zellij(self) -> bool {
		self == Self::Sixel
			|| (env_exists(ZELLIJ_KITTY_PASSTHROUGH_ENV)
				&& matches!(self, Self::Kgp | Self::KgpOld))
	}
}

#[cfg(test)]
mod tests {
	use std::{env, ffi::OsString, sync::Mutex};

	use super::*;

	static ENV_LOCK: Mutex<()> = Mutex::new(());

	struct EnvVarGuard {
		name: &'static str,
		old:  Option<OsString>,
	}

	impl EnvVarGuard {
		fn set(name: &'static str, value: &str) -> Self {
			let old = env::var_os(name);
			unsafe { env::set_var(name, value) };
			Self { name, old }
		}

		fn remove(name: &'static str) -> Self {
			let old = env::var_os(name);
			unsafe { env::remove_var(name) };
			Self { name, old }
		}
	}

	impl Drop for EnvVarGuard {
		fn drop(&mut self) {
			unsafe {
				if let Some(value) = self.old.take() {
					env::set_var(self.name, value);
				} else {
					env::remove_var(self.name);
				}
			}
		}
	}

	#[test]
	fn zellij_uses_sixel_by_default() {
		let _lock = ENV_LOCK.lock().unwrap();
		let _session = EnvVarGuard::set(ZELLIJ_SESSION_NAME_ENV, "test-session");
		let _passthrough = EnvVarGuard::remove(ZELLIJ_KITTY_PASSTHROUGH_ENV);

		assert!(Adapter::Sixel.supported_in_zellij());
		assert!(!Adapter::Kgp.supported_in_zellij());
		assert!(!Adapter::KgpOld.supported_in_zellij());
		assert!(!Adapter::Iip.supported_in_zellij());
	}

	#[test]
	fn zellij_kitty_passthrough_adds_kitty_adapters_only() {
		let _lock = ENV_LOCK.lock().unwrap();
		let _session = EnvVarGuard::set(ZELLIJ_SESSION_NAME_ENV, "test-session");
		let _passthrough = EnvVarGuard::set(ZELLIJ_KITTY_PASSTHROUGH_ENV, "1");

		assert!(Adapter::Sixel.supported_in_zellij());
		assert!(Adapter::Kgp.supported_in_zellij());
		assert!(Adapter::KgpOld.supported_in_zellij());
		assert!(!Adapter::Iip.supported_in_zellij());
	}
}
