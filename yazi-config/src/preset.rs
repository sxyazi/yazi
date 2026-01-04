use crate::{Yazi, keymap::Keymap, theme::Theme, vfs::Vfs};

pub(crate) struct Preset;

impl Preset {
	pub(super) fn yazi() -> Result<Yazi, toml::de::Error> {
		toml::from_str(&yazi_macro::config_preset!("yazi"))
	}

	pub(super) fn keymap() -> Result<Keymap, toml::de::Error> {
		toml::from_str(&yazi_macro::config_preset!("keymap"))
	}

	pub(super) fn theme(light: bool) -> Result<Theme, toml::de::Error> {
		toml::from_str(&if light {
			yazi_macro::theme_preset!("light")
		} else {
			yazi_macro::theme_preset!("dark")
		})
	}

	pub(super) fn vfs() -> Result<Vfs, toml::de::Error> {
		toml::from_str(&yazi_macro::config_preset!("vfs"))
	}

	#[inline]
	pub(crate) fn mix<E, A, B, C>(a: A, b: B, c: C) -> impl Iterator<Item = E>
	where
		A: IntoIterator<Item = E>,
		B: IntoIterator<Item = E>,
		C: IntoIterator<Item = E>,
	{
		a.into_iter().chain(b).chain(c)
	}
}
