use std::{env, ops::{Deref, DerefMut}};

use tracing::warn;
use yazi_emulator::{Emulator, TMUX};
use yazi_shared::env_exists;

use crate::drivers::{Driver as D, Ueberzug};

pub(crate) struct Drivers(Vec<D>);

impl Deref for Drivers {
	type Target = Vec<D>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for Drivers {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl From<&yazi_emulator::Emulator> for Drivers {
	fn from(value: &yazi_emulator::Emulator) -> Self { value.kind.either_into() }
}

impl From<yazi_emulator::Brand> for Drivers {
	fn from(value: yazi_emulator::Brand) -> Self {
		use yazi_emulator::Brand as B;

		Self(match value {
			B::Kitty => vec![D::Kgp],
			B::Konsole => vec![D::KgpOld],
			B::Iterm2 => vec![D::Iip, D::Sixel],
			B::WezTerm => vec![D::Iip, D::Sixel],
			B::Foot => vec![D::Sixel],
			B::Ghostty => vec![D::Kgp],
			B::Microsoft => vec![D::Sixel],
			B::Warp => vec![D::Iip, D::KgpOld],
			B::Rio => vec![D::Kgp],
			B::BlackBox => vec![D::Sixel],
			B::VSCode => vec![D::Iip, D::Sixel],
			B::Tabby => vec![D::Iip, D::Sixel],
			B::Hyper => vec![D::Iip, D::Sixel],
			B::Mintty => vec![D::Iip],
			B::Tmux => vec![],
			B::VTerm => vec![],
			B::Apple => vec![],
			B::Urxvt => vec![],
			B::Bobcat => vec![D::Iip, D::Sixel],
		})
	}
}

impl From<yazi_emulator::Unknown> for Drivers {
	fn from(value: yazi_emulator::Unknown) -> Self {
		Self(match (value.kgp, value.sixel) {
			(true, true) => vec![D::Sixel, D::KgpOld],
			(true, false) => vec![D::KgpOld],
			(false, true) => vec![D::Sixel],
			(false, false) => vec![],
		})
	}
}

impl Drivers {
	pub fn matches(emulator: &Emulator) -> D {
		let mut adapters: Self = emulator.into();
		if env_exists("ZELLIJ_SESSION_NAME") {
			adapters.retain(|p| *p == D::Sixel);
		} else if TMUX.get() {
			adapters.retain(|p| *p != D::KgpOld);
		}
		if let Some(p) = adapters.first() {
			return *p;
		}

		let supported_compositor = Ueberzug::supported_compositor();
		match env::var("XDG_SESSION_TYPE").unwrap_or_default().as_str() {
			"x11" => return D::X11,
			"wayland" if supported_compositor => return D::Wayland,
			"wayland" if !supported_compositor => return D::Chafa,
			_ => warn!("[Adapter] Could not identify XDG_SESSION_TYPE"),
		}
		if env_exists("WAYLAND_DISPLAY") {
			return if supported_compositor { D::Wayland } else { D::Chafa };
		}
		match env::var("DISPLAY").unwrap_or_default().as_str() {
			s if !s.is_empty() && !s.contains("/org.xquartz") => return D::X11,
			_ => {}
		}

		warn!("[Adapter] Falling back to chafa");
		D::Chafa
	}
}
