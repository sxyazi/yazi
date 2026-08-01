use std::{env, ops::{Deref, DerefMut}};

use tracing::warn;
use yazi_emulator::{Brand, Emulator};
use yazi_shared::env_exists;

use crate::drivers::{Driver as D, Ueberzug};

pub struct Drivers(Vec<D>);

impl Deref for Drivers {
	type Target = Vec<D>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for Drivers {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl From<&Emulator> for Drivers {
	fn from(value: &Emulator) -> Self {
		match value.brand {
			Brand::Unknown => Self(match (value.kgp, value.sixel) {
				(true, true) => vec![D::Sixel, D::KgpOld],
				(true, false) => vec![D::KgpOld],
				(false, true) => vec![D::Sixel],
				(false, false) => vec![],
			}),
			brand => brand.into(),
		}
	}
}

impl From<Brand> for Drivers {
	fn from(value: Brand) -> Self {
		use Brand as B;

		Self(match value {
			B::Unknown => vec![],
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

impl Drivers {
	pub fn matches(emu: &Emulator) -> D {
		let mut drivers: Self = emu.into();
		if env_exists("ZELLIJ_SESSION_NAME") {
			drivers.retain(|p| *p == D::Sixel);
		} else if emu.sixel && emu.mux.is_some_and(|mux| mux.sixel) {
			return D::Sixel;
		} else if emu.mux.is_some() {
			drivers.retain(|p| *p != D::KgpOld);
		}
		if let Some(d) = drivers.first() {
			return *d;
		}

		let supported_compositor = Ueberzug::supported_compositor();
		match env::var("XDG_SESSION_TYPE").unwrap_or_default().as_str() {
			"x11" => return D::X11,
			"wayland" if supported_compositor => return D::Wayland,
			"wayland" if !supported_compositor => return D::Chafa,
			_ => warn!("[Drivers] Could not identify XDG_SESSION_TYPE"),
		}
		if env_exists("WAYLAND_DISPLAY") {
			return if supported_compositor { D::Wayland } else { D::Chafa };
		}
		match env::var("DISPLAY").unwrap_or_default().as_str() {
			s if !s.is_empty() && !s.contains("/org.xquartz") => return D::X11,
			_ => {}
		}

		warn!("[Drivers] Falling back to chafa");
		D::Chafa
	}
}
