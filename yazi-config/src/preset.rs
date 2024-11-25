use std::{borrow::Cow, path::{Path, PathBuf}, str::FromStr};

use anyhow::{Context, Result};
use toml::{Table, Value};
use yazi_shared::Xdg;

use crate::theme::Flavor;

pub(crate) struct Preset;

impl Preset {
	pub(crate) fn yazi(p: &Path) -> Result<Cow<'static, str>> {
		Self::merge_path(p.join("yazi.toml"), yazi_macro::config_preset!("yazi"))
	}

	pub(crate) fn keymap(p: &Path) -> Result<Cow<'static, str>> {
		Self::merge_path(p.join("keymap.toml"), yazi_macro::config_preset!("keymap"))
	}

	pub(crate) fn flavor(light: bool, merge: bool) -> Result<Cow<'static, str>> {
		let theme = if merge {
			std::fs::read_to_string(Xdg::config_dir().join("theme.toml")).unwrap_or_default()
		} else {
			Default::default()
		};

		let flavor = Flavor::from_str(&theme)?;

		let preset =
			if light { yazi_macro::theme_preset!("light") } else { yazi_macro::theme_preset!("dark") };

		Self::merge_str(&theme, &Self::merge_str(&flavor.read(light)?, &preset)?)
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

	#[inline]
	pub(crate) fn merge_str(user: &str, base: &str) -> Result<Cow<'static, str>> {
		let mut t = user.parse()?;
		Self::merge(&mut t, base.parse()?, 2);

		Ok(t.to_string().into())
	}

	#[inline]
	fn merge_path(user: PathBuf, base: Cow<str>) -> Result<Cow<str>> {
		let s = std::fs::read_to_string(&user).unwrap_or_default();
		if s.is_empty() {
			return Ok(base);
		}

		Self::merge_str(&s, &base).with_context(|| format!("failed to parse config: {user:?}"))
	}

	fn merge(a: &mut Table, b: Table, max: u8) {
		for (k, v) in b {
			let Some(a) = a.get_mut(&k) else {
				a.insert(k, v);
				continue;
			};

			if max <= 1 {
				continue;
			}

			if let Some(a) = a.as_table_mut() {
				if let Value::Table(b) = v {
					Self::merge(a, b, max - 1);
					continue;
				}
			}
			*a = v;
		}
	}
}
