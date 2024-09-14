use std::{borrow::Cow, path::{Path, PathBuf}};

use anyhow::{anyhow, Context, Result};
use toml::{Table, Value};

use crate::{preset, theme::Flavor};

pub(crate) struct Preset;

impl Preset {
	pub(crate) fn yazi(p: &Path) -> Result<Cow<str>> {
		Self::merge_path(p.join("yazi.toml"), preset!("yazi"))
	}

	pub(crate) fn keymap(p: &Path) -> Result<Cow<str>> {
		Self::merge_path(p.join("keymap.toml"), preset!("keymap"))
	}

	pub(crate) fn theme(p: &Path) -> Result<Cow<str>> {
		let Ok(user) = std::fs::read_to_string(p.join("theme.toml")) else {
			return Ok(preset!("theme"));
		};
		let Some(use_) = Flavor::parse_use(&user) else {
			return Self::merge_str(&user, &preset!("theme"));
		};

		let p = p.join(format!("flavors/{use_}.yazi/flavor.toml"));
		let flavor =
			std::fs::read_to_string(&p).with_context(|| anyhow!("Failed to load flavor {p:?}"))?;

		Self::merge_str(&user, &Self::merge_str(&flavor, &preset!("theme"))?)
	}

	#[inline]
	pub(crate) fn mix<T, E>(a: T, b: T, c: T) -> impl Iterator<Item = E>
	where
		T: IntoIterator<Item = E>,
	{
		b.into_iter().chain(a).chain(c)
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

		Self::merge_str(&s, &base).with_context(|| anyhow!("Loading {user:?}"))
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
